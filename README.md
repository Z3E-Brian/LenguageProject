# 📘 ElementScript — README Oficial

**ElementScript** es un lenguaje de programación interpretado con sintaxis inspirada en conceptos químicos.
Cuenta con análisis léxico, parser Pratt, verificación semántica, generación de bytecode y ejecución mediante máquina virtual, además de un entorno gráfico (IDE) para pruebas interactivas.

Este README sirve como guía técnica completa del lenguaje y como documentación principal del repositorio.

---

# 🚀 Características principales

* 🔹 **Compilado a bytecode** mediante máquina virtual (VM)
* 🔹 **Tipado estático**
* 🔹 **Sintaxis inspirada en química**
* 🔹 **Ámbitos anidados (scopes)**
* 🔹 **Colecciones genéricas** (`solution<T>`, `sample<T>`)
* 🔹 **Entrada interactiva** con `capture`
* 🔹 **Errores con línea y columna**
* 🔹 **IDE gráfico integrado**

---

# 🧬 Tipos de Datos

## Tipos primitivos

| Clásico     | ElementScript | Ejemplo      |
| ----------- | ------------- | ------------ |
| int         | `atom_num`    | `3`          |
| float       | `mass`        | `3.14`       |
| string      | `formula`     | `"hola"`     |
| char        | `symbol`      | `'A'`        |
| bool        | `polarized`   | `pos`, `neg` |
| void/null   | `VoidState`   | `VoidState`  |

## Tipos compuestos

* `solution<T>` → vector dinámico
* `sample<T>` → lista dinámica

Ejemplos:

```elementscript
atom edad : atom_num = 20;
atom saludo : formula = "Hola";
atom inicial : symbol = 'A';
atom activo : polarized = pos;
```

---

# 🔧 Declaración de Variables

### Variables mutables — `atom`

```elementscript
atom contador : atom_num = 0;
contador = contador + 1;
```

### Constantes — `ion`

```elementscript
ion PI : mass = 3.14159;
PI = 4.0;      !! ERROR: no se puede reasignar
```

---

# ➕ Operadores

### Aritméticos

`+`, `-`, `*`, `/`, `%`

### Comparación

`==`, `!=`, `<`, `<=`, `>`, `>=`

### Lógicos

`and`, `or`, `not`

Ejemplo:

```elementscript
pos and neg
not pos
```

---

# 🔀 Estructuras de Control

## Condicionales (`itest`, `inotest`, `notest`)

```elementscript
itest (edad > 18) {
    emitln("Adulto");
} inotest (edad == 18) {
    emitln("Justo 18");
} notest {
    emitln("Menor de edad");
}
```

---

# 🔁 Bucles

### 1. `chain N` → Repetir N veces

```elementscript
chain 5 {
    emitln(chain.count());
}
```

### 2. `chain A to B`

```elementscript
chain 1 to 5 {
    emitln(chain.count());
}
```

### 3. Descendente

```elementscript
chain 5 to 1 {
    emitln(chain.count());
}
```

### 4. Estilo while (implementado en tu compilador)

```elementscript
atom i : atom_num = 0;
chain i < 5 > loop {
    emitln(i);
    i = i + 1;
}
```

---

# 🧪 Funciones (reaction)

Declaración:

```elementscript
reaction sumar(a : atom_num, b : atom_num) : atom_num {
    release a + b;
}
```

Uso:

```elementscript
atom r : atom_num = sumar(3, 4);
emitln(r);
```

---

# 📦 Colecciones

## solution<T> — vector dinámico

```elementscript
atom numeros : solution<atom_num> = [1, 2, 3];

numeros.push(10);
numeros.insert(1, 99);
numeros.remove(0);

emitln(numeros[0]);
emitln(numeros);
```

## sample<T> — lista dinámica

```elementscript
atom nombres : sample<formula> = ["Ana", "Luis"];

nombres.push_front("Zoe");
nombres.push_back("Carlos");

emitln(nombres[0]);
```

---

# ⌨ Entrada / Salida

## Salida

```elementscript
emit("Hola ");
emitln("mundo");
emitln("Resultado: ", 10);
```

## Entrada

```elementscript
atom nombre : formula;
capture nombre : formula;

emitln("Hola ", nombre);
```

---

# 📝 Comentarios

```elementscript
!! Comentario de una línea
```

---

# ⚠ Errores comunes y soluciones

### ❌ Reasignar un `ion`

```elementscript
ion x : atom_num = 5;
x = 10;     !! ERROR
```

### ❌ Usar variables en `chain`

```elementscript
ion N : atom_num = 5;
chain N { }    !! ERROR
```

Solución:

```elementscript
chain 5 { ... }               !! fijo
chain i < N > whileLoop { ... }  !! estilo while
```

### ❌ Índice fuera de rango

```elementscript
emitln(numeros[99]);   !! runtime error
```

### ❌ Tipos incompatibles

```elementscript
atom x : formula = "hola";
atom y : atom_num = x + 5;    !! ERROR
```

---

# 🧩 Ejemplos Completos

## Cuadrado 5×5 con `chain`

```elementscript
chain 1 to 5 {
    chain 1 to 5 {
        emit("■ ");
    }
    emit("\n");
}
```

## Factorial

```elementscript
reaction factorial(n : atom_num) : atom_num {
    itest (n <= 1) {
        release 1;
    } notest {
        release n * factorial(n - 1);
    }
}

atom r : atom_num = factorial(5);
emitln("5! = ", r);
```

---

# 🏗 Arquitectura Interna

ElementScript se compone de las siguientes etapas:

1. **Lexer** — Convierte texto → tokens
2. **Parser (Pratt)** — tokens → AST
3. **Analyzer** — Verifica tipos, constantes, ámbito
4. **Codegen** — AST → bytecode
5. **Executor** — máquina virtual que ejecuta bytecode
6. **GUI** — entorno de pruebas en tiempo real

---

# 🧑‍💻 Repositorio

**GitHub:** `Z3E-Brian/LenguageProject`
**Branch principal de desarrollo:** `develop`

---

# 📅 Información

* **Última actualización:** Noviembre 2025

---