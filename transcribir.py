#!/usr/bin/env python3

import argparse
import os
import sys
import time
from pathlib import Path

from faster_whisper import WhisperModel


def configurar_cuda():
    """
    Configura automáticamente las librerías CUDA/cuDNN instaladas
    mediante los paquetes NVIDIA de pip.

    Algunas versiones recientes de los paquetes NVIDIA tienen
    __file__ = None, por lo que no usamos nvidia.cublas.lib.__file__.
    """

    import site
    import glob

    rutas = []

    # Directorios típicos donde pip instala las librerías NVIDIA.
    posibles_roots = []

    try:
        posibles_roots.extend(site.getsitepackages())
    except Exception:
        pass

    try:
        posibles_roots.append(site.getusersitepackages())
    except Exception:
        pass

    # Buscar físicamente las librerías.
    patrones = [
        "nvidia/cublas/lib",
        "nvidia/cudnn/lib",
        "nvidia/cuda_runtime/lib",
    ]

    for root in posibles_roots:
        if not root:
            continue

        for patron in patrones:
            ruta = os.path.join(root, patron)

            if os.path.isdir(ruta):
                rutas.append(ruta)

    # Eliminar duplicados manteniendo el orden.
    rutas = list(dict.fromkeys(rutas))

    if not rutas:
        print("ERROR: No encuentro las librerías CUDA/cuDNN.")
        print()
        print("Comprueba que instalaste:")
        print()
        print("  pip install nvidia-cublas-cu12")
        print('  pip install "nvidia-cudnn-cu12==9.*"')
        print()
        sys.exit(1)

    # Añadir las rutas a LD_LIBRARY_PATH.
    actual = os.environ.get("LD_LIBRARY_PATH", "")

    nuevas = ":".join(rutas)

    if actual:
        os.environ["LD_LIBRARY_PATH"] = f"{nuevas}:{actual}"
    else:
        os.environ["LD_LIBRARY_PATH"] = nuevas

    print("Bibliotecas CUDA encontradas:")
    for ruta in rutas:
        print(f"  {ruta}")

    print()


def srt_timestamp(seconds):
    """Convierte segundos a formato SRT."""

    milliseconds = int(round(seconds * 1000))

    hours = milliseconds // 3_600_000
    milliseconds %= 3_600_000

    minutes = milliseconds // 60_000
    milliseconds %= 60_000

    secs = milliseconds // 1000
    milliseconds %= 1000

    return f"{hours:02}:{minutes:02}:{secs:02},{milliseconds:03}"


def es_final_de_frase(texto):
    """
    Determina si el texto parece terminar una frase.

    Robustez: ignora comillas/paréntesis y espacios finales,
    así cierra también si el texto termina en `...` o `."`.
    """
    texto = texto.strip()

    if not texto:
        return False

    # Quitar comillas/paréntesis de cierre del final.
    while texto and texto[-1] in '"\u201d\u2019\u201c\u2018\')]':
        texto = texto[:-1].strip()

    if not texto:
        return False

    # Puntos suspensivos también cuentan como final.
    if texto.endswith("..."):
        return True

    return texto[-1] in ".!?;:"


def fusionar_segmentos(segmentos, max_duration=15.0):
    """
    Fusiona segmentos de Whisper para crear bloques más naturales.

    Intenta:
    - Mantener bloques alrededor de max_duration segundos.
    - No cortar frases innecesariamente.
    - Respetar pausas naturales.
    """

    segmentos = list(segmentos)

    if not segmentos:
        return []

    resultado = []

    actual = {
        "start": segmentos[0].start,
        "end": segmentos[0].end,
        "text": segmentos[0].text.strip(),
    }

    palabras_debil = (
        "y",
        "o",
        "pero",
        "que",
        "porque",
        "como",
        "cuando",
        "si",
        "de",
        "del",
        "en",
        "con",
        "para",
        "por",
        "a",
    )

    pausa_min = 1.5

    for segmento in segmentos[1:]:

        texto = segmento.text.strip()

        if not texto:
            continue

        duracion_actual = actual["end"] - actual["start"]

        texto_actual = actual["text"].strip()

        # Pausa notable: mejor cerrar el bloque aquí y no
        # alargarlo cruzando un silencio largo.
        pausa = segmento.start - actual["end"]

        if pausa >= pausa_min and actual["text"].strip():

            resultado.append(actual)

            actual = {
                "start": segmento.start,
                "end": segmento.end,
                "text": texto,
            }

            continue

        # Si el bloque ya alcanzó el máximo,
        # preferimos cerrarlo en una frase.
        if duracion_actual >= max_duration:

            # Si ya terminó una frase, cerramos.
            if es_final_de_frase(texto_actual):

                resultado.append(actual)

                actual = {
                    "start": segmento.start,
                    "end": segmento.end,
                    "text": texto,
                }

                continue

            # Si la frase actual termina en una palabra fuerte,
            # también podemos cerrar.
            ultima_palabra = (
                texto_actual.rstrip(" ,")
                .split()[-1]
                .lower()
                .strip("¿¡.,;:!?")
            )

            if ultima_palabra not in palabras_debil:

                resultado.append(actual)

                actual = {
                    "start": segmento.start,
                    "end": segmento.end,
                    "text": texto,
                }

                continue

        # Si la unión no supera demasiado el máximo,
        # seguimos acumulando.
        actual["text"] += " " + texto
        actual["end"] = segmento.end

    # Último segmento.
    if actual["text"].strip():
        resultado.append(actual)

    return resultado


def crear_parrafos(segmentos, max_paragraph_duration=45.0):
    """
    Crea párrafos naturales para el TXT.

    Un párrafo puede contener varios bloques SRT.

    Reglas para empezar un párrafo nuevo:
    - Hay una pausa notable entre bloques (cambio de tema/hablante).
    - Llevamos bastante tiempo y hemos terminado una frase.
    - Superamos un tope duro (duración o caracteres) aunque no
      haya puntuación, para no crear párrafos gigantes.
    """
    if not segmentos:
        return []

    pausa_min = 1.5
    tope_duracion_hard = max_paragraph_duration * 2
    tope_caracteres = 2000

    parrafos = []

    actual = {
        "start": segmentos[0]["start"],
        "end": segmentos[0]["end"],
        "text": segmentos[0]["text"].strip(),
    }

    for segmento in segmentos[1:]:

        duracion = segmento["end"] - actual["start"]
        pausa = segmento["start"] - actual["end"]

        texto_actual = actual["text"].strip()
        numero_caracteres = len(texto_actual)

        cerrar = False

        # 1. Pausa notable: tema o turno nuevo.
        if pausa >= pausa_min:
            cerrar = True

        # 2. Ya llevamos tiempo y la frase quedó cerrada.
        elif (
            duracion >= max_paragraph_duration / 2
            and es_final_de_frase(texto_actual)
        ):
            cerrar = True

        # 3. Tope duro: no dejar crecer un párrafo sin salida.
        elif (
            duracion >= tope_duracion_hard
            or numero_caracteres >= tope_caracteres
        ):
            cerrar = True

        if cerrar:

            parrafos.append(actual)

            actual = {
                "start": segmento["start"],
                "end": segmento["end"],
                "text": segmento["text"].strip(),
            }

        else:

            actual["text"] += " " + segmento["text"].strip()
            actual["end"] = segmento["end"]

    if actual["text"].strip():
        parrafos.append(actual)

    return parrafos


def guardar_txt(segmentos, archivo):
    """
    Guarda la transcripción como texto con párrafos naturales.
    """

    parrafos = crear_parrafos(segmentos)

    with open(archivo, "w", encoding="utf-8") as f:

        for parrafo in parrafos:
            texto = parrafo["text"].strip()

            if texto:
                f.write(texto + "\n\n")


def guardar_srt(segmentos, archivo):
    """
    Guarda los segmentos agrupados como SRT.
    """

    with open(archivo, "w", encoding="utf-8") as f:

        for numero, segmento in enumerate(segmentos, start=1):

            texto = segmento["text"].strip()

            if not texto:
                continue

            inicio = srt_timestamp(segmento["start"])
            fin = srt_timestamp(segmento["end"])

            f.write(f"{numero}\n")
            f.write(f"{inicio} --> {fin}\n")
            f.write(f"{texto}\n\n")


def main():

    parser = argparse.ArgumentParser(
        description="Transcripción local con Faster-Whisper + CUDA"
    )

    parser.add_argument("audio", help="Archivo de audio/video a transcribir")

    parser.add_argument(
        "--modelo",
        default="turbo",
        choices=[
            "tiny",
            "base",
            "small",
            "medium",
            "large-v3",
            "turbo",
        ],
        help="Modelo Whisper. Por defecto: turbo",
    )

    parser.add_argument(
        "--idioma", default="es", help="Idioma del audio. Por defecto: es"
    )

    parser.add_argument(
        "--beam-size", type=int, default=5, help="Beam size. Por defecto: 5"
    )

    parser.add_argument("--sin-vad", action="store_true", help="Desactiva VAD")

    parser.add_argument(
        "--max-segment-duration",
        type=float,
        default=15.0,
        help="Duración máxima aproximada de cada bloque SRT. Por defecto: 15 segundos"
    )

    args = parser.parse_args()

    audio = Path(args.audio).expanduser().resolve()

    if not audio.exists():
        print(f"ERROR: No existe el archivo:")
        print(f"  {audio}")
        sys.exit(1)

    configurar_cuda()

    print()
    print("=" * 60)
    print("       FASTER-WHISPER / CUDA")
    print("=" * 60)
    print()
    print(f"Archivo : {audio}")
    print(f"Modelo  : {args.modelo}")
    print(f"Idioma  : {args.idioma}")
    print(f"GPU     : CUDA")
    print(f"Precisión: float16")
    print()

    print("Cargando modelo...")
    inicio_total = time.time()

    try:
        model = WhisperModel(
            args.modelo,
            device="cuda",
            compute_type="float16",
            device_index=0,
        )

    except Exception as e:
        print()
        print("ERROR AL INICIALIZAR CUDA")
        print("=" * 60)
        print(e)
        print()
        print("Comprueba que:")
        print("  1. nvidia-smi funciona.")
        print("  2. El driver NVIDIA está correctamente instalado.")
        print("  3. CUDA/cuDNN están instalados.")
        print()
        sys.exit(1)

    print("Modelo cargado.")
    print()
    print("Transcribiendo...")
    print("-" * 60)

    inicio_transcripcion = time.time()

    try:
        segmentos_generator, info = model.transcribe(
            str(audio),
            language=args.idioma,
            beam_size=args.beam_size,
            vad_filter=not args.sin_vad,
            condition_on_previous_text=True,
            word_timestamps=False,
        )

        # Es importante convertir el generator a lista.
        segmentos = list(segmentos_generator)

        segmentos = fusionar_segmentos(
            segmentos,
            max_duration=args.max_segment_duration
        )

    except Exception as e:
        print()
        print("ERROR DURANTE LA TRANSCRIPCIÓN")
        print("=" * 60)
        print(e)
        sys.exit(1)

    tiempo_transcripcion = time.time() - inicio_transcripcion

    print()
    print("-" * 60)
    print("Transcripción terminada.")
    print()

    print(f"Idioma detectado : {info.language}")
    print(f"Probabilidad     : {info.language_probability:.2%}")
    print(f"Duración audio   : {info.duration:.2f} segundos")
    print(f"Tiempo empleado  : {tiempo_transcripcion:.2f} segundos")

    if tiempo_transcripcion > 0:
        velocidad = info.duration / tiempo_transcripcion
        print(f"Velocidad        : {velocidad:.2f}x tiempo real")

    # ---------------------------------------------------------
    # Archivos de salida
    # ---------------------------------------------------------

    txt_file = audio.with_suffix(".txt")
    srt_file = audio.with_suffix(".srt")

    guardar_txt(segmentos, txt_file)
    guardar_srt(segmentos, srt_file)

    tiempo_total = time.time() - inicio_total

    print()
    print("=" * 60)
    print("                 COMPLETADO")
    print("=" * 60)
    print()
    print(f"TXT: {txt_file}")
    print(f"SRT: {srt_file}")
    print()
    print(f"Tiempo total: {tiempo_total:.2f} segundos")
    print()
    print("Los archivos se guardaron junto al audio.")
    print()


if __name__ == "__main__":
    main()
