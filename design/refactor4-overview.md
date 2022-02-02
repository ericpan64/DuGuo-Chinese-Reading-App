# DuGuo - Refactor 4

## Refactor Goals

### Make things Rust-y
There are a lot of areas that can be simplified by consolidating more code into Rust. For example, the way Redis is being used is just for storing static data (parsed CEDICT data) and that can be reasonably just stored in memory as a dict/map. Additionally, the way I'm using the NLP service doesn't leverage much and mainly does tokenization + NER, and I'm guessing there's an equivalent library in Rust that can capture that. This would really simplify the stack and make it more deployable, and more opportunities to build in Rust!

### Support Serverless Deployment
The idea of Serverless is appealing though could lead to vendor lock which is not appealing. A good middle ground would be: abstract the code files so it _could_ be deployed as serverless functions, however add an API framework for local dev. This could be a good opportunity to try actix-web.

### Try a WASM-based Frontend, again
Trying component-based frontends like Yew and React were interesting though possibly too much for my use-case (and a lot of work to re-architect from a template-based system that I'm on). However the idea of just bundling the frontend to a single file is pretty cool. Try something like Sycamore and see how that goes.

## Refactor Timeline

No timeline, though will aim to get something viable this year (2022)!