# Vendored sherpa-onnx runtime libraries

`libsherpa-onnx-c-api.so` y `libsherpa-onnx-cxx-api.so` se envían tal cual desde el
tarball oficial del release de sherpa-onnx. `libonnxruntime.so` es **reemplazado**
por la versión correcta de onnxruntime (ver abajo).

## Procedencia

| Archivo | Fuente | Comentario |
|---|---|---|
| `libsherpa-onnx-c-api.so` | `sherpa-onnx-v1.13.4-linux-x64-shared-lib.tar.bz2` (release `v1.13.4` de k2-fsa/sherpa-onnx) | Igual que el tarball. RPATH `$ORIGIN`, NEEDED `libonnxruntime.so` |
| `libsherpa-onnx-cxx-api.so` | mismo tarball | NEEDED `libsherpa-onnx-c-api.so` + `libonnxruntime.so` |
| `libonnxruntime.so` | `onnxruntime-linux-x64-1.27.0.tgz` (release `v1.27.0` de microsoft/onnxruntime) | Copia de `lib/libonnxruntime.so.1.27.0` renombrada |

URLs de descarga:

- `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.4/sherpa-onnx-v1.13.4-linux-x64-shared-lib.tar.bz2`
- `https://github.com/microsoft/onnxruntime/releases/download/v1.27.0/onnxruntime-linux-x64-1.27.0.tgz`

## Por qué el reemplazo de onnxruntime

sherpa-onnx 1.13.4 se compila contra la API 27 de onnxruntime, pero el tarball
`shared-lib` adjunta `libonnxruntime.so` 1.24.2 (API 24). Cargarlo produce:

```
The requested API version [27] is not available, only API versions [1, 24] are supported
```

Por eso el binario debe cargar `libonnxruntime.so` **1.27.0**. `libonnxruntime.so`
tiene SONAME interno `libonnxruntime.so.1`, pero `libsherpa-onnx-c-api.so` declara
NEEDED por el nombre de archivo `libonnxruntime.so` (sin versionar), de modo que el
loader lo resuelve por nombre de archivo. El RPM/deb se instalan por nombre de
archivo también, así que el nombre plano es correcto.

`ort`/`speakrs` con feature `load-dynamic` también buscan `libonnxruntime.so` vía
`dlopen`; `libsherpa-onnx-c-api.so` lleva `RPATH $ORIGIN`, por lo que la resolución
es siempre relativa al directorio donde estén los `.so`.

## Actualizar la versión

1. Bajar el nuevo tarball `shared-lib` de sherpa-onnx y copiar `libsherpa-onnx-c-api.so`
   y `libsherpa-onnx-cxx-api.so`.
2. Verificar contra qué API de ORT está compilado: `readelf -d libsherpa-onnx-c-api.so | grep NEEDED`
   y confirmar la versión de onnxruntime que necesita (comprobar
   `SHERPA_ONNX_VERSION` / notas del release, o `strings` del binario).
3. Bajar el onnxruntime correspondiente desde
   `https://github.com/microsoft/onnxruntime/releases` y copiar su
   `lib/libonnxruntime.so.*` renombrado a `libonnxruntime.so`.
4. Actualizar las rutas en `tauri.conf.json` (`bundle.resources`) y este README.

## Empaquetado / release

- `tauri.conf.json` → `bundle.resources` empaqueta estos `.so` en
  `usr/lib/<product>/` (deb: `/usr/lib/subtitledss/`, rpm: `/usr/lib/subtitledss/`,
  AppImage: `usr/lib/subtitledss/`).
- `src-tauri/.cargo/config.toml` enlaza el binario con
  `-Wl,-rpath,$ORIGIN/../lib/subtitledss` + `-Wl,--disable-new-dtags`. Se usa
  RPATH (DT_RPATH) y no RUNPATH porque `ort`/`speakrs` cargan vía `dlopen`, que
  ignora DT_RUNPATH. El binario está en `usr/bin/`, por lo que
  `$ORIGIN/../lib/subtitledss` resuelve en los tres formatos.
- `linux.deb.depends` y `linux.rpm.depends` apuntan a los paquetes **runtime**
  (`libasound2`, `libpipewire-0.3-0` / `alsa-lib`, `pipewire-libs`), no a los
  `-dev`; tauri agrega automáticamente webkit2gtk/gtk/appindicator.

### AppImage en Arch Linux

El linuxdeploy que baja tauri tiene un `strip` viejo que no soporta secciones
RELR ("unknown type [0x13] section `.relr.dyn'") y su plugin gtk asume un
directorio de loaders de gdk-pixbuf estilo Debian que no existe en Arch (los
loaders van embebidos en la librería). En Arch usar:

```sh
./scripts/build-appimage-arch.sh
```

(el script aplica `NO_STRIP=1` y parchea el plugin gtk del caché de tauri;
deb/rpm no se ven afectados). En Debian/Ubuntu (CI) el AppImage se genera sin
workarounds.

## Verificación

```sh
readelf -d vendor/sherpa/libsherpa-onnx-c-api.so | grep -E "NEEDED|RPATH"
readelf -d vendor/sherpa/libonnxruntime.so | grep SONAME
readelf -d target/release/subtitledss | grep -E "RPATH|RUNPATH"
```

## Licencias

- sherpa-onnx: Apache-2.0 (`https://github.com/k2-fsa/sherpa-onnx`)
- onnxruntime: MIT (`https://github.com/microsoft/onnxruntime`)
