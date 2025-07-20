# Rust Macros to Unify Synchronous and Asynchronous Codebases

The async/await mechanism has become a useful tool for handling asynchronous programming in I/O-bound applications. However, it introduces a significant challenge: async code is "contagious," meaning it cannot be used in synchronous contexts. Consequently, a developer who creates a library must provide two implementations: synchronous and asynchronous. A significant portion of the asynchronous implementation, which doesn’t require concurrency, ends up being redundant, describing exactly the same logic in a slightly different syntax.

This library offers two macros, `#[_async]` and `_await!(...)`, to address the issue of managing the code that must exist in both synchronous and asynchronous versions. These macros allow you to annotate such code, and the library will generate both the synchronous and asynchronous versions automatically. For example, the following code:

```rust
use build_async::*;

#[_async]
fn foo() {
    _await!(boo());
    _await!(x.zoo());
}
```

Will expand to:

```rust
fn foo() {
    boo();
    x.zoo();
}
async fn foo_async() {
    boo_async().await;
    x.zoo_async().await;
}
```
The `#[_async]` macro can be applied to any non-async function or method definition, whether inside or outside of `impl` or `trait` definitions. This macro generates both synchronous and asynchronous versions of the function. The `_await!(...)` macro can be applied function or method calls within the function or method definitions marked with `#[_async]`. Depending on the context (synchronous or asynchronous), `_await!(...)` expands to either a synchronous call or an asynchronous call using `await`. To avoid name conflicts, the name of the asynchronous version produced by this mechanism is given the postfix `_async`.

Here is a more comprehensive example that demonstrates how these macros can be used within `trait` and `impl` definitions:

```rust
use build_async::*;

trait Writer {
    type Error; 

    #[_async]
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        unimplemented!()
    }
}                   
                    
impl<T: std::io::Write> Writer for T {
    type Error = std::io::Error;

    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.write(self, bytes)
    }
}

impl<T: tokio::io::AsyncWriteExt> Writer for T {
    type Error = tokio::io::Error;

    async fn write_async(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.write(self, bytes).await
    }
}

trait Encoder {
    type Error;

    #[_async]
    fn encode_bool(&mut self, value: &bool) -> Result<(), Self::Error> {
        _await!(self.encode_u8(&(*value).into()))
    }   
    ...
}

trait Encode {
    #[_async]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error>;
}   

impl<T: Encode> Encode for Box<T> {
    #[_async]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        _await!(T::encode(self, encoder))?;
        Ok(())
    }
}
```

The `_await` macro assumes the presence of both synchronous and asynchronous versions of the function it is applied to, with the asynchronous version being async and differing in name only by the `_async` postfix. If these conventions aren't followed in the existing codebase, adapters like `Writer` can be implemented to bridge the gap.

For real-life examples, refer to crates [cerdito](https://crates.io/crates/cerdito) and [rustbif](https://crates.io/crates/rustbif).

Finally, there is one peculiar detail about this library: We recommend importing macros implicitly using `use build_async::*;`. Otherwise, the explicit import statement will have to look like this: `use build_async::{_async, _await_sync, _await_async};`. This can be confusing because it raises questions like "Where is `_await`?" and "Why do I need to import `_await_sync` and `_await_async`?". The reason is that `_await` is a "pseudo-macro." It looks and feels like a macro, but it has never been defined. When the `_async` macro encounters `_await`, it replaces it with `_await_sync` or `_await_async`, depending on the context.

