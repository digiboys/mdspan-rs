Implement a faithful Rust version of C++ `mdspan` (based on P0009 / `std::mdspan`), with a Rust-native API where necessary, but preserving the core model and semantics.

High-level goals:
- `mdspan` is a non-owning multidimensional view over existing storage.
- Prefer a `NonNull<T>`-based internal representation with explicit lifetimes via `PhantomData`.
- Provide separate shared and mutable view types:
  - `MdSpan<'a, T, M = LayoutRightMapping<...>, A = DefaultAccessor<T>>`
  - `MdSpanMut<'a, T, M = LayoutRightMapping<...>, A = DefaultAccessor<T>>`
- The implementation should be sound with respect to Rust aliasing and lifetimes.

Core model to preserve from C++:
- Separate the concepts of:
  1. element type
  2. extents
  3. layout policy
  4. mapping
  5. accessor
- Do not collapse layout and mapping into one concept. Layout policy should produce or correspond to a mapping type; `MdSpan` should store the mapping object.
- Mapping is responsible for:
  - extents
  - multidimensional index -> linear offset
  - `required_span_size`
  - layout properties like uniqueness / exhaustiveness / stridedness
  - stride queries where applicable
- Accessor is responsible for:
  - data handle type
  - reference-like result type
  - `access`
  - `offset`
- For this Rust version, you may omit a distinct `OffsetPolicy` associated type and assume the offset policy is always `Self`.

Representation:
- Prefer storing:
  - `NonNull<T>` as the default data handle for normal accessors
  - mapping object
  - accessor object
  - `PhantomData<&'a T>` or `PhantomData<&'a mut T>` as appropriate
- `MdSpan` should model shared access.
- `MdSpanMut` should model mutable access.

Extents:
- Rust lacks variadic generics, so choose a practical maximum supported rank, e.g. 12, for static/mixed-static extents support.
- Support:
  - fully dynamic extents
  - static extents up to the chosen maximum rank
  - mixed static/dynamic extents up to the chosen maximum rank
- Be explicit and consistent about where this maximum rank limit applies.
- Rank should be compile-time when practical.
- Extents API should expose rank and per-dimension extent.

Layout and mapping:
- Implement standard layout policies and mappings analogous to C++:
  - `LayoutRight`
  - `LayoutLeft`
  - `LayoutStride`
- Mapping types should store the state they need, including extents and strides where applicable.
- `LayoutStride` mapping should support arbitrary strides and correct `required_span_size`.
- Preserve the conceptual split:
  - layout policy = family/category
  - mapping = concrete mapping instance

Accessor:
- Define an `Accessor` trait in the C++ spirit, but adapted to Rust.
- `Accessor` should include:
  - associated `Element`
  - associated `Pointer`
  - associated `Reference<'a>`
  - `fn access<'a>(&self, p: &'a Self::Pointer, i: usize) -> Self::Reference<'a>`
  - `fn offset(&self, p: Self::Pointer, i: usize) -> Self::Pointer`
- Assume the offset policy is always `Self`; do not add a separate `OffsetPolicy` associated type unless clearly needed internally.
- Be careful with lifetime design:
  - `access` should not unnecessarily tie the output lifetime to `&self`
  - `offset` should return an owned/rebased handle without output lifetime coupling
- Implement at least:
  - `DefaultAccessor<T>`
  - an example `ConstantAccessor<T>` whose handle may itself be the constant value and whose `access` ignores the index
  - an example `ScaledAccessor<T>` that stores a runtime scale factor and returns scaled values on access

Mutability and soundness:
- Shared access may yield `&T` for the default accessor.
- Mutable access may yield `&mut T` only where sound.
- Be conservative about soundness for `MdSpanMut`, especially for non-unique mappings.
- Construction and APIs should enforce or document the invariants required for sound mutable access.

Construction:
- Provide convenient builders for both shared and mutable spans:
  - defaults should be ergonomic for the common case
  - allow overriding layout and accessor when needed
- Desired style:
  - easy default construction from slices / mutable slices
  - optional customization of layout and accessor via builder methods
- Example builder goals:
  - `MdSpan::builder(...)`
  - `MdSpanMut::builder(...)`
  - default layout = `LayoutRight`
  - default accessor = `DefaultAccessor`
  - ability to swap in `LayoutLeft`, `LayoutStride`, `ConstantAccessor`, `ScaledAccessor`, etc.

API surface:
- Provide checked and unchecked element access.
- Provide rank/extents/introspection APIs.
- Provide `required_span_size`.
- Provide stride queries where meaningful.
- Avoid overengineering beyond what is needed for a faithful, robust core.

Implementation guidance:
- Favor a clean, well-factored design over maximal cleverness.
- Use `unsafe` only where necessary, and document each unsafe invariant clearly.
- Include comments explaining how the Rust design corresponds to the C++ `mdspan` model.
- Include a small set of tests/examples covering:
  - dynamic extents
  - static extents
  - `LayoutRight`
  - `LayoutLeft`
  - `LayoutStride`
  - default accessor
  - constant accessor
  - scaled accessor
  - builder-based construction
  - shared and mutable access

Deliver:
- the core traits and types
- a coherent module structure
- representative tests/examples
- brief notes on any places where Rust limitations forced deviations from the C++ design

Before you go ahead and implement this, make sure to reference the official specification: https://eel.is/c++draft/views#multidim. This prompt may have contained errors or inconsistencies, prefer the specification.

Also, make sure you implement this separating things out cleanly into modules and creating fine grained bazel dependencies for them to optimize cache use in rebuilds.
