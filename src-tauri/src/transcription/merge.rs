use crate::asr::engine::TranscriptionSegment;

pub const PAUSA_MIN: f64 = 1.5;

const PALABRAS_DEBIL: &[&str] = &[
    "y", "o", "pero", "que", "porque", "como", "cuando", "si", "de", "del", "en", "con", "para",
    "por", "a",
];

pub trait TimedText {
    fn start(&self) -> f64;
    fn end(&self) -> f64;
    fn text(&self) -> &str;
}

impl TimedText for TranscriptionSegment {
    fn start(&self) -> f64 {
        self.start
    }

    fn end(&self) -> f64 {
        self.end
    }

    fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergeSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

impl TimedText for MergeSegment {
    fn start(&self) -> f64 {
        self.start
    }

    fn end(&self) -> f64 {
        self.end
    }

    fn text(&self) -> &str {
        &self.text
    }
}

/// Determina si el texto parece terminar una frase.
///
/// Robustez: ignora comillas/paréntesis y espacios finales,
/// así cierra también si el texto termina en `...` o `."`.
pub fn es_final_de_frase(texto: &str) -> bool {
    let mut t = texto.trim();

    if t.is_empty() {
        return false;
    }

    // Quitar comillas/paréntesis de cierre del final.
    loop {
        let Some(ultimo) = t.chars().last() else {
            return false;
        };

        if matches!(
            ultimo,
            '"' | '\u{201d}' | '\u{2019}' | '\u{201c}' | '\u{2018}' | ')' | ']'
        ) {
            t = t[..t.len() - ultimo.len_utf8()].trim();
        } else {
            break;
        }
    }

    if t.is_empty() {
        return false;
    }

    if t.ends_with("...") {
        return true;
    }

    matches!(t.chars().last().unwrap(), '.' | '!' | '?' | ';' | ':')
}

/// Fusiona segmentos de Whisper para crear bloques más naturales.
///
/// Intenta:
/// - Mantener bloques alrededor de `max_duration` segundos.
/// - No cortar frases innecesariamente.
/// - Respetar pausas naturales.
pub fn fusionar_segmentos<T: TimedText>(segmentos: &[T], max_duration: f64) -> Vec<MergeSegment> {
    if segmentos.is_empty() {
        return Vec::new();
    }

    let mut resultado: Vec<MergeSegment> = Vec::new();

    let mut actual = MergeSegment {
        start: segmentos[0].start(),
        end: segmentos[0].end(),
        text: segmentos[0].text().trim().to_string(),
    };

    for segmento in &segmentos[1..] {
        let texto = segmento.text().trim().to_string();

        if texto.is_empty() {
            continue;
        }

        let duracion_actual = actual.end - actual.start;
        let texto_actual = actual.text.trim().to_string();

        // Pausa notable: mejor cerrar el bloque aquí y no
        // alargarlo cruzando un silencio largo.
        let pausa = segmento.start() - actual.end;

        if pausa >= PAUSA_MIN && !actual.text.trim().is_empty() {
            resultado.push(actual);

            actual = MergeSegment {
                start: segmento.start(),
                end: segmento.end(),
                text: texto,
            };

            continue;
        }

        // Si el bloque ya alcanzó el máximo,
        // preferimos cerrarlo en una frase.
        if duracion_actual >= max_duration {
            // Si ya terminó una frase, cerramos.
            if es_final_de_frase(&texto_actual) {
                resultado.push(actual);

                actual = MergeSegment {
                    start: segmento.start(),
                    end: segmento.end(),
                    text: texto,
                };

                continue;
            }

            // Si la frase actual termina en una palabra fuerte,
            // también podemos cerrar.
            let ultima_palabra = texto_actual
                .trim_end_matches([' ', ','])
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_lowercase()
                .trim_matches(['¿', '¡', '.', ',', ';', ':', '!', '?'])
                .to_string();

            if !PALABRAS_DEBIL.contains(&ultima_palabra.as_str()) {
                resultado.push(actual);

                actual = MergeSegment {
                    start: segmento.start(),
                    end: segmento.end(),
                    text: texto,
                };

                continue;
            }
        }

        // Si la unión no supera demasiado el máximo,
        // seguimos acumulando.
        actual.text.push(' ');
        actual.text.push_str(&texto);
        actual.end = segmento.end();
    }

    // Último segmento.
    if !actual.text.trim().is_empty() {
        resultado.push(actual);
    }

    resultado
}

/// Crea párrafos naturales para el TXT.
///
/// Un párrafo puede contener varios bloques SRT.
///
/// Reglas para empezar un párrafo nuevo:
/// - Hay una pausa notable entre bloques (cambio de tema/hablante).
/// - Llevamos bastante tiempo y hemos terminado una frase.
/// - Superamos un tope duro (duración o caracteres) aunque no
///   haya puntuación, para no crear párrafos gigantes.
pub fn crear_parrafos(
    segmentos: &[MergeSegment],
    max_paragraph_duration: f64,
) -> Vec<MergeSegment> {
    if segmentos.is_empty() {
        return Vec::new();
    }

    let tope_duracion_hard = max_paragraph_duration * 2.0;
    let tope_caracteres = 2000;

    let mut parrafos: Vec<MergeSegment> = Vec::new();

    let mut actual = MergeSegment {
        start: segmentos[0].start,
        end: segmentos[0].end,
        text: segmentos[0].text.trim().to_string(),
    };

    for segmento in &segmentos[1..] {
        let duracion = segmento.end - actual.start;
        let pausa = segmento.start - actual.end;

        let texto_actual = actual.text.trim().to_string();
        let numero_caracteres = texto_actual.chars().count();

        let cerrar = pausa >= PAUSA_MIN
            || (duracion >= max_paragraph_duration / 2.0 && es_final_de_frase(&texto_actual))
            || duracion >= tope_duracion_hard
            || numero_caracteres >= tope_caracteres;

        if cerrar {
            parrafos.push(actual);

            actual = MergeSegment {
                start: segmento.start,
                end: segmento.end,
                text: segmento.text.trim().to_string(),
            };
        } else {
            actual.text.push(' ');
            actual.text.push_str(segmento.text.trim());
            actual.end = segmento.end;
        }
    }

    if !actual.text.trim().is_empty() {
        parrafos.push(actual);
    }

    parrafos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start: f64, end: f64, text: &str) -> MergeSegment {
        MergeSegment {
            start,
            end,
            text: text.to_string(),
        }
    }

    #[test]
    fn es_final_detecta_puntuacion() {
        assert!(es_final_de_frase("Hola."));
        assert!(es_final_de_frase("¿Qué?"));

        assert!(!es_final_de_frase("Hola"));
        assert!(!es_final_de_frase(""));
        assert!(!es_final_de_frase("   "));
    }

    #[test]
    fn es_final_detecta_ellipsis_y_cierres() {
        assert!(es_final_de_frase("..."));
        assert!(es_final_de_frase("Entro."));
        assert!(es_final_de_frase("\"cierra.\""));
        assert!(es_final_de_frase("hola)."));
        assert!(!es_final_de_frase("bien)"));
    }

    #[test]
    fn fusiona_segmentos_cortos() {
        let entrada = vec![seg(0.0, 1.0, "hola"), seg(1.2, 2.0, "mundo")];
        let salida = fusionar_segmentos(&entrada, 15.0);

        assert_eq!(salida.len(), 1);
        assert_eq!(salida[0].text, "hola mundo");
        assert_eq!(salida[0].start, 0.0);
        assert_eq!(salida[0].end, 2.0);
    }

    #[test]
    fn fusiona_cierra_en_pausa_notable() {
        let entrada = vec![seg(0.0, 1.0, "hola"), seg(3.0, 4.0, "mundo")];
        let salida = fusionar_segmentos(&entrada, 15.0);

        assert_eq!(salida.len(), 2);
        assert_eq!(salida[0].text, "hola");
        assert_eq!(salida[1].text, "mundo");
    }

    #[test]
    fn fusiona_cierra_en_frase_cuando_supera_maximo() {
        let entrada = vec![
            seg(0.0, 16.0, "esta es una frase."),
            seg(16.0, 17.0, "hola"),
        ];
        let salida = fusionar_segmentos(&entrada, 15.0);

        assert_eq!(salida.len(), 2);
        assert_eq!(salida[0].text, "esta es una frase.");
        assert_eq!(salida[1].text, "hola");
    }

    #[test]
    fn fusiona_no_corta_tras_palabra_debil() {
        let entrada = vec![
            seg(0.0, 16.0, "esto es algo que"),
            seg(16.0, 17.0, "continuaba"),
        ];
        let salida = fusionar_segmentos(&entrada, 15.0);

        assert_eq!(salida.len(), 1);
        assert_eq!(salida[0].text, "esto es algo que continuaba");
    }

    #[test]
    fn fusiona_cierra_tras_palabra_fuerte_al_maximo() {
        let entrada = vec![
            seg(0.0, 16.0, "yo canto el viernes"),
            seg(16.0, 17.0, "algo"),
        ];
        let salida = fusionar_segmentos(&entrada, 15.0);

        assert_eq!(salida.len(), 2);
        assert_eq!(salida[0].text, "yo canto el viernes");
        assert_eq!(salida[1].text, "algo");
    }

    #[test]
    fn fusiona_ignora_vacios() {
        let entrada = vec![
            seg(0.0, 1.0, "hola"),
            seg(1.0, 1.5, "   "),
            seg(1.5, 2.0, "mundo"),
        ];
        let salida = fusionar_segmentos(&entrada, 15.0);

        assert_eq!(salida.len(), 1);
        assert_eq!(salida[0].text, "hola mundo");
    }

    #[test]
    fn parrafos_separan_con_pausa() {
        let entrada = vec![seg(0.0, 1.0, "bloque uno"), seg(3.0, 4.0, "bloque dos")];
        let salida = crear_parrafos(&entrada, 45.0);

        assert_eq!(salida.len(), 2);
        assert_eq!(salida[0].text, "bloque uno");
        assert_eq!(salida[1].text, "bloque dos");
    }

    #[test]
    fn parrafos_acumulan_dentro_de_limites() {
        let entrada = vec![seg(0.0, 5.0, "primero"), seg(5.1, 10.0, "segundo")];
        let salida = crear_parrafos(&entrada, 45.0);

        assert_eq!(salida.len(), 1);
        assert_eq!(salida[0].text, "primero segundo");
        assert_eq!(salida[0].start, 0.0);
        assert_eq!(salida[0].end, 10.0);
    }

    #[test]
    fn parrafos_cierran_por_tope_duro() {
        let entrada = vec![
            seg(0.0, 95.0, "un bloque bien largo"),
            seg(95.1, 96.0, "siguiente"),
        ];
        let salida = crear_parrafos(&entrada, 45.0);

        assert_eq!(salida.len(), 2);
    }

    #[test]
    fn parrafos_cierran_tras_media_duracion_con_frase() {
        let entrada = vec![
            seg(0.0, 30.0, "mitad de duracion cerrada."),
            seg(30.1, 35.0, "sigue"),
        ];
        let salida = crear_parrafos(&entrada, 45.0);

        assert_eq!(salida.len(), 2);
    }

    #[test]
    fn parrafos_vacia_entrada() {
        assert!(crear_parrafos(&[], 45.0).is_empty());
    }
}
