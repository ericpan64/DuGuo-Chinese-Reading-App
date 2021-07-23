# DuGuo - Refactor 4

## Refactor Goals
- Switch UI to use Elm + Materal UI ([elm-mdc](https://github.com/aforemny/elm-mdc)) (SPA) instead of vanilla HTML + Bootstrap 5 + Tera Templates (MPA). Keep the core functionality, including:
    - Multi-page Routing
    - Word-level popups + selections
    - Appropriate POST calls and API routes
- It compiles

## Refactor "Nice-to-Haves"
- Re-evaluate usability of certain features (i.e. by end of refactor, close #17)
- Make website more modern which entails:
    - Consistent and multi-color scheme
    - Interactivity with components
- Add Duey (对龙)! Incorporate into each page in a smart way.
- Consider how "Article Scores" would be displayed in the application
- Consider configuration on how the website/pop-ups render
    - E.g. multiple color schemes, popup size, font size, available buttons, etc.