# json-stream
A library to parse Newline Delimited JSON values from a byte stream.

The motivating use-case for this crate is parsing newline-delimited json values from Axum's [BodyStream](https://docs.rs/axum/latest/axum/extract/struct.BodyStream.html), but can be used with any `Stream<Item = Result<B, E>>` where `B: Deref<Target = [u8]>`.

## Memory usage

`JsonStream` is designed to minimize the usage of memory to be able to handle any sized stream. When JSON values are deserialized, the bytes are deleted from the underlying buffer. This is to make room for the next chunk of bytes we'll read in when necessary.  

## Example

```rust
use json_stream::JsonStream;
use futures::stream::once;
use futures::StreamExt;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Foo {
    bar: String,
}

#[tokio::main]
async fn main() {
    // This could be any stream that yields bytes, such as a file stream or a network stream.
    let pinned_bytes_future = Box::pin(async {
        Ok::<_, std::io::Error>(r#"{"bar": "foo"}\n{"bar": "qux"}\n{"bar": "baz"}"#.as_bytes())
    });
    let mut json_stream = JsonStream::<Foo, _>::new(once(pinned_bytes_future));
    
    while let Some(Ok(foo)) = json_stream.next().await {
        println!("{:?}", foo);
    }
}
```
