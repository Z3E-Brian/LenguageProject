# ElementScript

**ElementScript** es un lenguaje de programación interpretado diseñado e implementado desde cero en Rust, con una sintaxis inspirada en terminología química. El proyecto implementa un pipeline de compilación completo: análisis léxico, análisis sintáctico (parser Pratt), verificación semántica con sistema de tipos, generación de bytecode y ejecución mediante una máquina virtual de pila propia. Incluye además un IDE gráfico integrado construido con `egui`.

> Proyecto académico desarrollado en la Universidad Nacional de Costa Rica (UNA) como trabajo de diseño e implementación de lenguajes de programación.

---

## Características técnicas

- **Pipeline completo de compilación** — Lexer → Parser → Análisis semántico → Codegen → VM
- **Parser Pratt (Precedence Climbing)** — manejo correcto de precedencia y asociatividad de operadores
- **Sistema de tipos estático** con inferencia parcial y verificación en tiempo de compilación
- **Máquina virtual de pila** con soporte para scopes anidados, funciones y llamadas recursivas
- **Colecciones genéricas** — vector dinámico (`solution<T>`) y lista dinámica (`sample<T>`)
- **Bucles con acceso al contador** — `chain N` y `chain A to B` con variable de iteración automática (`loop_1`, `loop_2`, ...)
- **Bucle condicional** `orbite` (equivalente a `while`)
- **Entrada interactiva** — instrucción `capture` con soporte para pausa/reanudación de la VM
- **Reporte de errores** con línea y columna en las tres fases (léxico, sintáctico, semántico)
- **IDE gráfico** con modo oscuro/claro, snippets predefinidos y ejecución en tiempo real

---

## Arquitectura

```
Código fuente (.elem)
        │
        ▼
  ┌─────────────┐
  │    Lexer    │  → Vec<Token>                  lexer.rs
  └─────────────┘
        │
        ▼
  ┌─────────────┐
  │   Parser    │  → AST (Vec<Stmt>)             parser.rs
  │   (Pratt)   │
  └─────────────┘
        │
        ▼
  ┌─────────────────┐
  │ Semantic Pass   │  → errores de tipos/scope  semantics.rs
  └─────────────────┘
        │
        ▼
  ┌──────────────┐
  │  Code Gen    │  → Vec<Instruction>           codegen.rs
  └──────────────┘
        │
        ▼
  ┌──────────────┐
  │   Executor   │  → String (output)            executor.rs
  │   (Stack VM) │
  └──────────────┘
```

Los módulos de utilidades (`utils/`) contienen los tipos compartidos entre etapas: tokens, nodos del AST, instrucciones de bytecode y valores en tiempo de ejecución.

---

## Estructura del repositorio

```
src/
├── main.rs          # Pipeline principal y modo CLI
├── lexer.rs         # Analizador léxico
├── parser.rs        # Parser Pratt + AST
├── semantics.rs     # Verificación semántica y sistema de tipos
├── codegen.rs       # Generador de bytecode
├── executor.rs      # Máquina virtual de pila
├── gui.rs           # IDE gráfico (egui / eframe)
└── utils/
    ├── enums.rs     # TokenKind, Stmt, Expr, Instruction, Value, Ty
    ├── token.rs     # Struct Token + Display
    └── utils.rs     # Tabla de keywords
```

---

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) ≥ 1.75

---

## Compilación y ejecución

```bash
# Clonar el repositorio
git clone https://github.com/Z3E-Brian/LenguageProject.git
cd LenguageProject

# Modo GUI (IDE interactivo)
cargo run

# Modo CLI (ejecutar un archivo .elem)
cargo run -- programa.elem

# Build optimizado
cargo build --release
./target/release/elementscript programa.elem
```

---

## El lenguaje

ElementScript usa terminología inspirada en química. La siguiente tabla mapea los conceptos clásicos a los del lenguaje:

| Concepto       | Clásico      | ElementScript       |
|----------------|--------------|---------------------|
| Variable       | `var`        | `atom`              |
| Constante      | `const`      | `ion`               |
| Función        | `function`   | `reaction`          |
| Retorno        | `return`     | `release`           |
| If             | `if`         | `itest`             |
| Else if        | `else if`    | `inotest`           |
| Else           | `else`       | `notest`            |
| For / loop     | `for`        | `chain`             |
| While          | `while`      | `orbite`            |
| Entero         | `int`        | `atom_num`          |
| Float          | `float`      | `mass`              |
| String         | `string`     | `formula`           |
| Char           | `char`       | `symbol`            |
| Bool           | `bool`       | `polarized`         |
| true / false   | `true/false` | `pos / neg`         |
| void           | `void`       | `VoidState`         |
| Vector         | `Vec<T>`     | `solution<T>`       |
| Lista          | `LinkedList` | `sample<T>`         |
| Comentario     | `//`         | `!!`                |

---

## Sintaxis y ejemplos

### Variables y constantes

```elem
atom edad    : atom_num  = 25;
atom nombre  : formula   = "Brian";
atom activo  : polarized = pos;
ion  PI      : mass      = 3.14159;
```

### Condicionales

```elem
itest (edad >= 18) {
    emitln("Mayor de edad");
} inotest (edad == 17) {
    emitln("Casi adulto");
} notest {
    emitln("Menor de edad");
}
```

### Bucles

```elem
!! Repetir N veces (con contador automático loop_1)
chain 5 {
    emitln("Iteración: ", loop_1);
}

!! Rango ascendente o descendente
chain 1 to 10 {
    emitln(loop_1);
}

chain 10 to 1 {
    emitln(loop_1);
}

!! Bucles anidados (loop_1 externo, loop_2 interno)
chain 1 to 3 {
    chain 1 to 3 {
        emit(loop_1, "x", loop_2, " ");
    }
    emit("\n");
}

!! Equivalente a while
atom i : atom_num = 0;
orbite (i < 5) {
    emitln(i);
    i = i + 1;
}
```

### Funciones

```elem
reaction factorial(n : atom_num) : atom_num {
    itest (n <= 1) {
        release 1;
    } notest {
        release n * factorial(n - 1);
    }
}

atom resultado : atom_num = factorial(6);
emitln("6! = ", resultado);
```

### Colecciones

```elem
!! Vector dinámico
atom numeros : solution<atom_num> = [1, 2, 3, 4, 5];
numeros.push(99);
numeros.set(0, 10);
emitln(numeros[0]);     !! → 10
emitln(numeros.len());  !! → 6

!! Lista dinámica
atom nombres : sample<formula> = ["Ana", "Luis"];
nombres.push_front("Zoe");
nombres.push_back("Carlos");
emitln(nombres[0]);     !! → Zoe
```

### Entrada de usuario

```elem
atom nombre : formula;
capture nombre;
emitln("Hola, ", nombre);
```

---

## Sistema de tipos

El verificador semántico (`semantics.rs`) implementa:

- **Compatibilidad numérica** — `atom_num` es promovible a `mass`
- **Verificación de mutabilidad** — las constantes (`ion`) no pueden reasignarse
- **Verificación de funciones** — aridad y tipos de parámetros comprobados en sitio de llamada
- **Tipos genéricos** — `solution<T>` y `sample<T>` con verificación del tipo elemento en `push`, `set`, `insert`, etc.
- **Bucles correctamente tipados** — variables de contador (`loop_N`) declaradas automáticamente como `atom_num` de solo lectura en cada scope de bucle
- **Scopes léxicos anidados** con resolución correcta de nombres

---

## Bytecode y VM

El generador de código produce una lista lineal de instrucciones (`Vec<Instruction>`) que la VM ejecuta mediante un programa counter (`pc`). Las instrucciones incluyen:

- `LoadConst`, `LoadVar`, `StoreVar` — manejo del stack de valores
- `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Neg`, `Not` — operaciones aritméticas y lógicas
- `Equal`, `Less`, `LessEqual`, ... — comparaciones
- `Jump`, `JumpIfFalse`, `JumpIfTrue` — control de flujo (patching de direcciones en codegen)
- `Call`, `Return`, `PushScope`, `PopScope` — gestión de llamadas y scopes
- `EmitLn`, `Emit`, `Capture` — I/O
- `StartLoopCapture`, `EndLoopCapture` — implementación eficiente de `chain`
- Builtins para colecciones: `__vec_from`, `__vec_push`, `__vec_set`, `__vec_pop`, `__vec_get`, `__vec_len`, `__list_*`

---

## IDE

El IDE gráfico (construido con [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)) ofrece:

- Editor de código con botón de ejecución
- Panel de salida con los resultados del programa
- Soporte para entrada interactiva (`capture`) con pausa y reanudación de la VM
- Modo oscuro y modo claro
- Snippets predefinidos agrupados por categoría

---

## Tecnologías

| Herramienta | Uso |
|-------------|-----|
| Rust        | Lenguaje de implementación |
| egui / eframe | GUI del IDE |
| Cargo       | Gestión de dependencias y build |

---

## Autores

| Contributor | Área de trabajo |
|-------------|-----------------|
| [Z3E-Brian](https://github.com/Z3E-Brian) | Diseño del lenguaje, lexer, parser, análisis semántico, codegen, VM |
| [MarconiCalvo](https://github.com/MarconiCalvo) | IDE gráfico (egui), entrada interactiva en terminal, bucles anidados, UI/UX |

Proyecto desarrollado en la Universidad Nacional de Costa Rica (UNA).