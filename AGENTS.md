# Agent Guidelines

## Git Commit Conventions

All git commit messages in this repository must strictly follow the **Conventional Commits v1.0.0** specification.

### Format
```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

### Guidelines
- Use the imperative present tense in the description (e.g., `feat: add ...` instead of `feat: added ...`).
- Keep the first line concise (50–72 characters).
- Include relevant context or motivations in the body when necessary.
- **Line Count Verification**: Check the line counts of modified files only upon the prompt `create git message` to verify that no modified file exceeds the 200-line limit before generating the commit message.

---

## Architectural & Code Philosophy: Pure OOP (Alan Kay & Yegor Bugayenko)

All code in this project must strictly adhere to **Pure Object-Oriented Programming (OOP)** as conceptualized by **Alan Kay** and formulated by **Yegor Bugayenko (Elegant Objects)**.

### 1. Alan Kay's Foundation
- **Messaging over Data**: Objects communicate exclusively by sending and receiving messages. Never treat objects as passive data containers.
- **Encapsulation & Local Retention**: Internal state and processes are strictly encapsulated, private, and fully concealed from other objects.
- **Late Binding**: Behavior is bound dynamically at runtime via interfaces and polymorphism, not statically or via procedural branching.

### 2. Elegant Objects Rules (Yegor Bugayenko)
- **No Getters and Setters**: Never extract data from an object to process it outside ("Tell, Don't Ask"). Objects represent living entities with behavior, not bags of data.
- **No NULL**: Never use, accept, or return `null` / `nullptr` / `None`. Use Null Objects, Optional types, or polymorphism instead.
- **No Static Methods or Utility Classes**: No `Utils`, `Helpers`, or static procedural functions. Everything must be an instance of an object that implements an interface.
- **Immutability by Default**: Objects should be immutable whenever feasible. State transitions produce new object representations rather than mutating existing fields.
- **No Public Fields**: All attributes and state are strictly private.
- **No Type Casting or Type Inspection**: Never use `instanceof`, `dynamic_cast`, type checking, or reflection to determine behavior. Use polymorphism and method dispatch.
- **Interfaces Everywhere**: Code against clean, focused interfaces / traits. Objects must be decoupled and swappable.
- **Clean Constructors**: Primary constructors do nothing except assign arguments to internal fields. No I/O, heavy calculations, or side effects during construction.
- **Composition & Decorators**: Favor object composition and decorators over class inheritance.
- **Small Files (Max 200 Lines)**: No source code file may exceed 200 lines of code. Classes, traits, and modules must be small, highly cohesive, and focused on a single responsibility. Split larger files into dedicated, decoupled object files. Verification of modified files' line counts is performed upon the prompt `create git message`.
