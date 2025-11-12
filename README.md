# ElementScript Language 🔥

Un lenguaje de programación diseñado para la manipulación de bucles y emisión de texto, con una sintaxis minimalista y expresiva.

## 📋 Descripción

ElementScript es un lenguaje de programación interpretado que incluye:
- **Analizador léxico** (Lexer)
- **Analizador sintáctico** (Parser)
- **Análisis semántico** completo
- **Generador de código intermedio**
- **Ejecutor de bytecode**
- **Interfaz gráfica** (GUI) con egui/eframe

## 🚀 Características

- ✨ Sintaxis simple y clara
- 🔄 Bucles con la palabra clave `chain`
- 📝 Emisión de texto con `emit()`
- 🔢 Variables locales y arrays
- 🎨 Interfaz gráfica integrada para escribir y ejecutar código
- 💻 Soporte para línea de comandos
- 🛡️ Análisis semántico robusto con detección de errores

## 📦 Instalación

### Prerrequisitos
- Rust 1.70 o superior
- Cargo (incluido con Rust)

### Clonar el repositorio
```bash
git clone https://github.com/Z3E-Brian/LenguageProject.git
cd LenguageProject
```

### Compilar el proyecto
```bash
cargo build --release
```

## 🎮 Uso

### Modo GUI (Interfaz Gráfica)
Ejecuta el programa sin argumentos para abrir la interfaz gráfica:

```bash
cargo run
```

### Modo CLI (Línea de Comandos)
Ejecuta un archivo `.elem` directamente:

```bash
cargo run -- archivo.elem
```

O con el ejecutable compilado:

```bash
./target/release/LenguageProject archivo.elem
```

## 📝 Sintaxis de ElementScript

### Estructura básica

```elementscript
!! Esto es un comentario

!! Emisión de texto
emit("Hola, mundo!\n");

!! Bucles (chain)
chain 3 {
    emit("Iteración: ", loop_1, "\n");
}

!! Bucles anidados
chain 2 {
    emit("Nivel 1: ", loop_1, "\n");
    chain 3 {
        emit("  Nivel 2: ", loop_1, ", ", loop_2, "\n");
    }
}
```

### Variables

```elementscript
!! Declaración de variables
int x = 10;
str nombre = "ElementScript";

!! Uso de variables
emit("El valor de x es: ", x, "\n");
```

### Arrays

```elementscript
!! Declaración de arrays
int[5] numeros;

!! Asignación
numeros[0] = 100;
numeros[1] = 200;

!! Uso en bucles
chain 5 {
    numeros[loop_1 - 1] = loop_1 * 10;
}
```

### Palabras clave

- `chain` - Define un bucle
- `emit` - Emite texto a la salida
- `int` - Tipo de dato entero
- `str` - Tipo de dato cadena
- `loop_N` - Variable automática para el índice del bucle N (empezando desde 1)

## 📁 Estructura del Proyecto

```
LenguageProject/
├── src/
│   ├── main.rs          # Punto de entrada y pipeline de compilación
│   ├── lexer.rs         # Análisis léxico
│   ├── parser.rs        # Análisis sintáctico
│   ├── semantics.rs     # Análisis semántico
│   ├── codegen.rs       # Generador de código intermedio
│   ├── executor.rs      # Ejecutor de bytecode
│   ├── gui.rs           # Interfaz gráfica
│   └── utils/           # Utilidades (tokens, enums, etc.)
├── test_*.elem          # Archivos de prueba
├── Cargo.toml           # Configuración del proyecto
└── README.md            # Este archivo
```

## 🔧 Pipeline de Compilación

El proceso de compilación y ejecución sigue estos pasos:

1. **Análisis Léxico** - Convierte el código fuente en tokens
2. **Análisis Sintáctico** - Genera el AST (Abstract Syntax Tree)
3. **Análisis Semántico** - Verifica tipos, variables y reglas semánticas
4. **Generación de Código** - Produce código intermedio (bytecode)
5. **Ejecución** - Interpreta y ejecuta el bytecode

## 📊 Ejemplos

### Ejemplo 1: Hola Mundo
```elementscript
emit("Hola, ElementScript!\n");
```

### Ejemplo 2: Tabla de Multiplicar
```elementscript
chain 10 {
    chain 10 {
        int resultado = loop_1 * loop_2;
        emit(loop_1, " x ", loop_2, " = ", resultado, "\n");
    }
}
```

### Ejemplo 3: Pirámide de Asteriscos
```elementscript
chain 5 {
    chain loop_1 {
        emit("*");
    }
    emit("\n");
}
```

## 🧪 Pruebas

El proyecto incluye varios archivos de prueba:

- `test_simple.elem` - Prueba de bucles anidados básicos
- `test_single.elem` - Prueba de bucle simple
- `test_3_niveles.elem` - Prueba de bucles de 3 niveles
- `test_simple_no_nested.elem` - Prueba sin anidamiento
- `test_bucle_problema.elem` - Casos de prueba específicos

Ejecuta las pruebas con:
```bash
cargo run -- test_simple.elem
```

## 🛠️ Desarrollo

### Compilar en modo debug
```bash
cargo build
```

### Ejecutar con logs
```bash
cargo run -- archivo.elem
```

### Verificar el código
```bash
cargo check
cargo clippy
```

## 📜 Licencia

Este proyecto está bajo la licencia MIT. Consulta el archivo LICENSE para más detalles.

## 👤 Autor

**Brian Zeledón**
- GitHub: [@Z3E-Brian](https://github.com/Z3E-Brian)

## 🤝 Contribuciones

Las contribuciones son bienvenidas. Por favor:
1. Haz fork del proyecto
2. Crea una rama para tu característica (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## 📚 Recursos Adicionales

- [Documentación de Rust](https://www.rust-lang.org/learn)
- [egui Framework](https://github.com/emilk/egui)
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)

## 🗺️ Roadmap

- [ ] Soporte para funciones personalizadas
- [ ] Operaciones aritméticas más complejas
- [ ] Estructuras de control adicionales (if/else, while)
- [ ] Sistema de módulos
- [ ] Exportación a otros formatos
- [ ] REPL interactivo
- [ ] Depurador integrado

---

⭐ Si te gusta este proyecto, ¡dale una estrella en GitHub!
