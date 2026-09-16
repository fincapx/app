<div align="center">

# FinCapX

**Un gestor de finanzas personales local-first, riguroso por dentro y sencillo por fuera.**

[![Licencia: AGPL v3](https://img.shields.io/badge/licencia-AGPL--3.0-blue.svg)](LICENSE)
[![Estado](https://img.shields.io/badge/estado-en%20dise%C3%B1o-orange.svg)](docs/design.md)

</div>

---

> **Estado del proyecto: en diseño.** Todavía no hay código. El
> [documento de diseño](docs/design.md) define el modelo de datos y las decisiones
> fundamentales antes de escribir la primera línea.

## La idea

Las aplicaciones de finanzas personales libres se reparten en dos extremos. De un lado,
libros contables rigurosos como Beancount o GnuCash: exactos, auditables, y con una curva
de aprendizaje pensada para contadores. Del otro, aplicaciones cómodas como Actual Budget:
un placer de usar, pero con un modelo de datos que renuncia a la multi-moneda y a la
contabilidad formal.

FinCapX busca el punto que nadie ocupa: **partida doble de verdad por debajo, una
aplicación moderna por encima**. El usuario registra "gasté 50.000 en comida"; el programa
guarda un asiento contable que cuadra al céntimo y se puede auditar diez años después.

## Principios

- **Tus datos son tuyos.** Archivo SQLite local, formato abierto, exportable completo. Sin
  cuenta, sin registro, sin conexión obligatoria.
- **Exactitud antes que comodidad.** Importes en enteros, nunca en coma flotante. Si algo
  no cuadra, falla en voz alta.
- **Universal por defecto.** Ninguna decisión asume un país, una moneda o un idioma.
- **La complejidad se esconde, no se elimina.**

## Estado y hoja de ruta

| Fase | Contenido | Estado |
|---|---|---|
| 0 | Documento de diseño y modelo de datos | En curso |
| 1 | Núcleo contable en Rust + `verify` | Pendiente |
| 2 | Interfaz de escritorio (Tauri) | Pendiente |
| 3 | Multi-moneda, presupuestos, informes | Pendiente |
| 4 | Importar/exportar, traducciones | Pendiente |

Fuera de la primera versión, y a propósito: sincronización, IA, inversiones, conciliación
bancaria. Son módulos que se añaden encima del núcleo sin migrar datos.

## Plataformas previstas

Linux, macOS y Windows, desde una sola base de código (Tauri 2).

## Contribuir

El proyecto está en fase de diseño: ahora mismo la contribución más útil es **discutir el
[documento de diseño](docs/design.md)**. Si ves un error en el modelo contable, en la
representación del dinero o en el esquema, abre un issue — corregirlo hoy cuesta una
conversación; corregirlo con usuarios cuesta una migración.

## Licencia

[GNU AGPL-3.0](LICENSE). Puedes usar, estudiar, modificar y distribuir este software. Si lo
modificas y lo distribuyes —incluso ofreciéndolo como servicio en red— debes publicar tus
cambios bajo la misma licencia.
