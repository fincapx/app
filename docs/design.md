# FinCapX — Documento de diseño

> Estado: **borrador**. Escrito antes de la primera línea de código, a propósito.
> Última revisión: 2026-09-23

Este documento existe para que, dentro de seis meses, exista una respuesta escrita a la
pregunta "¿por qué está hecho así?". Todo lo que está aquí es discutible **hasta** que se
escriba código encima; lo marcado como *irreversible* deja de serlo en cuanto haya usuarios
con datos.

---

## 1. Qué es FinCapX

Un gestor de finanzas personales de escritorio, local-first, que cualquier persona del
mundo pueda usar sin adaptarlo a su país, su moneda o su forma de llevar cuentas.

**La tesis del proyecto, en una frase:** un libro contable riguroso por debajo, una
aplicación financiera moderna por encima.

El panorama actual deja un hueco claro:

| Proyecto | Rigor contable | Experiencia de usuario |
|---|---|---|
| Beancount / GnuCash | Alto | Para contadores |
| Actual Budget | Bajo (mono-moneda) | Excelente |
| Firefly III | Medio | Requiere servidor |

Nadie ocupa la esquina "riguroso **y** agradable". Ese es el objetivo.

### Qué no es

- No es una app de presupuesto por sobres (envelope budgeting) exclusivamente.
- No es un sistema de contabilidad para empresas.
- No es un servicio en la nube. La nube, si llega, será opcional.
- No es un cliente bancario. No se conecta a bancos ni descarga movimientos.

---

## 2. Principios

1. **Los datos son del usuario.** Archivo local, formato abierto, exportable completo en
   todo momento. Sin cuenta, sin registro, sin conexión obligatoria.
2. **Exactitud antes que comodidad.** Si una operación no cuadra, falla; no se redondea en
   silencio.
3. **La complejidad se esconde, no se elimina.** El usuario nunca escribe un asiento
   contable, pero el asiento existe y es auditable.
4. **Universal por defecto.** Ninguna decisión asume Colombia, ni el peso, ni el español.
5. **Aburrido por dentro.** SQLite, Rust, y dependencias que se puedan justificar una
   por una. No se trata de tener pocas, sino de que cada una sea estándar de facto en su
   ecosistema, esté mantenida, arrastre poco detrás de sí y no tenga avisos de seguridad
   abiertos. Una dependencia "exótica" no es la que hace algo raro: es la que nadie más
   usa, o la que no puedes auditar. Antes de añadir una al núcleo: `cargo tree` para ver
   qué trae consigo, y `cargo audit` para contrastarla con el registro RustSec.

---

## 3. Las cinco decisiones irreversibles

Son irreversibles porque cambiarlas después obliga a migrar los datos de todos los
usuarios, y una migración mal hecha en una app de dinero destruye la confianza de forma
permanente.

### 3.1 El dinero se representa como entero en unidades menores

Un importe es un `i64` que cuenta **unidades menores** de su moneda, acompañado del código
de la moneda. La escala vive en la tabla `currency`.

| Moneda | `minor_unit` | Valor guardado | Significa |
|---|---|---|---|
| COP | 0 | `1500` | $ 1.500 |
| USD | 2 | `1500` | $ 15.00 |
| BHD | 3 | `1500` | 1.500 BHD |

**Nunca se usa punto flotante para dinero.** Ni `f32`, ni `f64`, ni `REAL` en SQLite.
Money Manager Ex usa `double` y arrastra errores de redondeo por eso.

El rango de `i64` es de ±9,22·10¹⁸ unidades menores. Con dos decimales son noventa mil
billones de unidades mayores: suficiente incluso para monedas hiperinflacionadas.

**Las tasas de cambio no son dinero** y no siguen esta regla. Se guardan como decimal en
texto y se operan con precisión arbitraria (`rust_decimal`). Una tasa es un factor, no un
saldo.

### 3.2 Partida doble obligatoria

Toda transacción está compuesta por dos o más *postings* (apuntes). Cada posting mueve un
importe con signo en una cuenta.

Un gasto de 50.000 en comida pagado en efectivo no es "un registro de tipo gasto". Son dos
apuntes:

```
2026-09-16  "Almuerzo"
  Efectivo          -50000 COP
  Gastos:Comida     +50000 COP
```

**Las categorías son cuentas.** No existe una tabla `category` aparte: una categoría es una
cuenta de tipo `income` o `expense`. Esto es lo que permite que informes, presupuestos y
saldos salgan de una sola consulta en lugar de tres subsistemas que se contradicen.

El usuario nunca ve la palabra "apunte" ni escribe dos líneas. La interfaz pide "cuánto,
de dónde, en qué" y el núcleo construye los apuntes.

### 3.3 El invariante de balance es por moneda, y es exacto

> Dentro de una transacción, para cada moneda, la suma de los importes de sus apuntes es
> exactamente cero.

Sin tolerancias, sin épsilon, sin redondeo. Aritmética entera.

Esto obliga a modelar los cambios de divisa con **cuentas de intercambio** (el modelo de
*trading accounts* de GnuCash). Convertir 100 USD a 400.000 COP son cuatro apuntes:

```
2026-09-16  "Cambio de divisa"
  Cuenta USD            -10000 USD   (100.00)
  Intercambio:USD       +10000 USD
  Intercambio:COP      -400000 COP
  Cuenta COP           +400000 COP
```

USD suma cero. COP suma cero. La tasa implícita (1 USD = 4.000 COP) queda registrada en los
propios importes, y la cuenta de intercambio acumula la ganancia o pérdida cambiaria sin que
nadie tenga que calcularla.

La alternativa —convertir todo a una moneda base y tolerar diferencias de redondeo— es más
fácil de programar y produce libros que no cuadran. Se descarta.

El invariante se valida en el núcleo, no en SQL: SQLite no puede expresar una restricción
que abarque varias filas. Debe existir un comando `verify` que recorra todo el libro y lo
compruebe.

### 3.4 Identidad estable y borrado lógico

- **Identificadores:** UUIDv7 (RFC 9562), guardados como `TEXT`. Ordenables por tiempo como
  ULID, pero estándar y reconocibles por cualquier herramienta.
- **Generados en el cliente**, nunca autoincrementales. Un `INTEGER PRIMARY KEY` hace
  imposible sincronizar dos dispositivos.
- **Toda entidad sincronizable lleva** `created_at`, `updated_at` y `deleted_at`, en
  ISO-8601 UTC.
- **El borrado es lógico** (`deleted_at`). Un borrado físico no se puede propagar a otro
  dispositivo: si la fila desaparece, no hay nada que replicar y el dato revive en la
  siguiente sincronización.

Esto se decide ahora aunque la sincronización no exista todavía, porque es exactamente lo
que no se puede añadir después sin reescribir cada tabla.

### 3.5 Registro de cambios append-only

Existe una tabla `change_log` a la que solo se añade, nunca se modifica ni se borra. Cada
escritura en el modelo deja una entrada.

**Por qué esto y no *event sourcing* completo:** en event sourcing el estado actual no se
guarda, se reconstruye plegando los eventos. Es elegante y hace que cada consulta sea un
fold sobre la historia, que cada informe necesite proyecciones, y que un error en un evento
antiguo sea muy caro de corregir.

Aquí el estado actual vive en tablas normales —consultas rápidas, código simple— y el log
corre en paralelo. Se pierde la pureza y se gana que un `SELECT` sea un `SELECT`.

El log sirve para tres cosas, en este orden:

1. Auditoría: qué cambió, cuándo, desde qué dispositivo.
2. Deshacer.
3. Sincronización: cuando llegue, el log *es* el registro de replicación.

---

## 4. Esquema del núcleo

SQLite. `PRAGMA foreign_keys = ON` siempre. `PRAGMA journal_mode = WAL`.

```sql
-- Configuración de la instancia: schema_version, instance_id,
-- base_currency, locale, device_id.
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Catálogo de monedas. Se siembra con ISO 4217; el usuario puede añadir
-- monedas propias (cripto, puntos, tiempo) con la escala que quiera.
CREATE TABLE currency (
    code        TEXT PRIMARY KEY,          -- 'COP', 'USD', 'EUR'
    name        TEXT NOT NULL,
    symbol      TEXT NOT NULL,
    minor_unit  INTEGER NOT NULL CHECK (minor_unit BETWEEN 0 AND 8),
    is_custom   INTEGER NOT NULL DEFAULT 0
);

-- Cuentas y categorías. Jerárquicas vía parent_id.
CREATE TABLE account (
    id          TEXT PRIMARY KEY,
    parent_id   TEXT REFERENCES account(id),
    type        TEXT NOT NULL CHECK (
                    type IN ('asset','liability','equity',
                             'income','expense','trading')),
    name        TEXT NOT NULL,
    -- Obligatoria en cuentas reales; NULL en income/expense, que
    -- aceptan cualquier moneda.
    currency    TEXT REFERENCES currency(code),
    is_system   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    CHECK (type IN ('income','expense') OR currency IS NOT NULL)
);

-- 'transaction' es palabra reservada en SQL, de ahí 'txn'.
CREATE TABLE txn (
    id          TEXT PRIMARY KEY,
    date        TEXT NOT NULL,             -- YYYY-MM-DD
    payee       TEXT,
    narration   TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

-- El apunte contable. amount va con signo y en unidades menores.
CREATE TABLE posting (
    id          TEXT PRIMARY KEY,
    txn_id      TEXT NOT NULL REFERENCES txn(id) ON DELETE CASCADE,
    account_id  TEXT NOT NULL REFERENCES account(id),
    amount      INTEGER NOT NULL,
    currency    TEXT NOT NULL REFERENCES currency(code),
    memo        TEXT
);

-- Tasas para informes y conversiones sugeridas. Nunca altera importes
-- ya registrados.
CREATE TABLE fx_rate (
    date    TEXT NOT NULL,
    base    TEXT NOT NULL REFERENCES currency(code),
    quote   TEXT NOT NULL REFERENCES currency(code),
    rate    TEXT NOT NULL,                 -- decimal en texto
    source  TEXT NOT NULL,                 -- 'frankfurter', 'manual'
    PRIMARY KEY (date, base, quote)
);

-- Append-only. Sin UPDATE, sin DELETE, nunca.
CREATE TABLE change_log (
    seq        INTEGER PRIMARY KEY AUTOINCREMENT,
    entity     TEXT NOT NULL,
    entity_id  TEXT NOT NULL,
    op         TEXT NOT NULL CHECK (op IN ('insert','update','delete')),
    payload    TEXT NOT NULL,              -- JSON del estado resultante
    ts         TEXT NOT NULL,
    device_id  TEXT NOT NULL
);

CREATE INDEX idx_posting_account ON posting(account_id);
CREATE INDEX idx_posting_txn     ON posting(txn_id);
CREATE INDEX idx_txn_date        ON txn(date) WHERE deleted_at IS NULL;
CREATE INDEX idx_change_entity   ON change_log(entity, entity_id);
```

Eso es todo el núcleo. Presupuestos, metas, préstamos e informes son **módulos que se
apoyan encima**, con sus propias tablas, y ninguno puede alterar estas seis.

---

## 5. Alcance

### Dentro del MVP

- Configuración inicial: idioma, moneda, cuentas de partida.
- Cuentas y categorías jerárquicas.
- Registrar ingreso, gasto y transferencia (partida doble oculta).
- Multi-moneda con cuentas de intercambio.
- Historial con filtros y edición.
- Presupuestos mensuales por categoría.
- Informes básicos: patrimonio, flujo, gasto por categoría.
- Exportar e importar el libro completo.
- Comando `verify` de integridad contable.

### Fuera del MVP, explícitamente

Sincronización · IA · inversiones y cotizaciones · préstamos con amortización ·
conciliación bancaria · importación de CSV de bancos concretos · facturas · móvil ·
informes fiscales.

Cada una de estas es un módulo que se añade encima del núcleo sin migrar nada. Están fuera
por tiempo, no por arquitectura.

---

## 6. Decisiones aplazadas

**Cifrado en reposo.** No en el MVP. Es aplazable porque cifrar el archivo (SQLCipher o
equivalente) no cambia el modelo de datos: es una migración de formato, no de esquema. Lo
que sí se decide ya es que **ningún secreto se guarda en la base de datos** —las claves de
API van al llavero del sistema operativo.

**Sincronización.** El `change_log`, los UUIDv7 y las lápidas (`deleted_at`) son los
cimientos. El algoritmo concreto (CRDT, relojes híbridos, árboles de Merkle) se elige
cuando haya usuarios que lo pidan. Actual Budget resolvió esto bien y su enfoque está
documentado.

**IA.** Módulo con interfaz de proveedor intercambiable. Dos modos previstos: modelo local
y clave propia del usuario (BYOK). Una suscripción alojada es un traspaso de costes y exige
límites de uso desde el primer día; no está en los planes.

---

## 7. Licencia

**AGPL-3.0.**

Permite a cualquiera usar, estudiar, modificar y distribuir el software. Obliga a que
cualquier versión modificada se publique bajo la misma licencia — incluso si se ofrece como
servicio en red, que es la diferencia con la GPL normal.

El razonamiento: MIT permitiría que un tercero tome el proyecto, le añada sincronización de
pago y lo cierre, sin devolver nada. Con AGPL, quien construya encima contribuye de vuelta.
El coste es que impide ciertos modelos comerciales cerrados; se acepta a conciencia.

---

## 8. Estructura del repositorio

```
fincapx/app
├── Cargo.toml          workspace
├── crates/
│   └── core/           ledger, dominio, almacenamiento
├── desktop/            Tauri + interfaz
└── docs/
```

Se arranca como workspace de Cargo con un solo crate. Convertir un proyecto normal en
workspace después es molesto; añadir un crate a un workspace existente es una línea.

`core/` se irá partiendo (`money`, `ledger`, `storage`, `fx`) a medida que crezca. Regla
para saber cuándo: **un crate merece existir cuando puedes decir qué hace sin usar la
palabra "y"**.
