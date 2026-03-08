module Components.Markdown exposing (view)

import Html exposing (Html, code, div, pre, text)
import Html.Attributes exposing (attribute, class, style)
import Markdown.Parser
import Markdown.Renderer
import UI


view : String -> Html msg
view markdownString =
    case
        markdownString
            |> Markdown.Parser.parse
            |> Result.mapError (\_ -> "Failed to parse markdown")
            |> Result.andThen (Markdown.Renderer.render mermaidRenderer)
    of
        Ok rendered ->
            let
                codeBlockBg =
                    UI.colors.bgSurface

                codeBlockText =
                    UI.colors.textPrimary
            in
            div
                [ class "markdown-content"
                , style "color" UI.colors.textSecondary
                , style "font-size" "0.9375rem"
                , style "line-height" "1.7"
                , style "word-break" "break-word"
                ]
                (Html.node "style" []
                    [ text
                        (".markdown-content h1 { font-size: 1.5rem; font-weight: bold; margin-bottom: 1rem; margin-top: 1.5rem; color: "
                            ++ UI.colors.textPrimary
                            ++ "; }"
                            ++ ".markdown-content h2 { font-size: 1.25rem; font-weight: 600; margin-bottom: 0.75rem; margin-top: 1.25rem; color: "
                            ++ UI.colors.textPrimary
                            ++ "; }"
                            ++ ".markdown-content h3 { font-size: 1.125rem; font-weight: 500; margin-bottom: 0.5rem; margin-top: 1rem; color: "
                            ++ UI.colors.textPrimary
                            ++ "; }"
                            ++ ".markdown-content ul { list-style-type: disc; padding-left: 1.5rem; margin-bottom: 1rem; }"
                            ++ ".markdown-content ol { list-style-type: decimal; padding-left: 1.5rem; margin-bottom: 1rem; }"
                            ++ ".markdown-content li { margin-bottom: 0.25rem; }"
                            ++ ".markdown-content p { margin-bottom: 1rem; }"
                            ++ ".markdown-content p:last-child { margin-bottom: 0; }"
                            ++ ".markdown-content code { font-family: "
                            ++ UI.fontMono
                            ++ "; background-color: "
                            ++ codeBlockBg
                            ++ "; color: "
                            ++ codeBlockText
                            ++ "; padding: 0.125rem 0.25rem; border-radius: 0.25rem; font-size: 0.875rem; border: 1px solid "
                            ++ UI.colors.border
                            ++ "; }"
                            ++ ".markdown-content pre { background-color: "
                            ++ codeBlockBg
                            ++ "; color: "
                            ++ codeBlockText
                            ++ "; padding: 1rem; border-radius: 0.375rem; overflow-x: auto; margin-bottom: 1rem; border: 1px solid "
                            ++ UI.colors.border
                            ++ "; }"
                            ++ ".markdown-content pre code { background-color: transparent; border: 0; padding: 0; font-size: 0.875rem; }"
                            ++ ".markdown-content a { color: "
                            ++ UI.colors.accent
                            ++ "; text-decoration: underline; }"
                            ++ ".markdown-content strong { font-weight: 700; color: "
                            ++ UI.colors.textPrimary
                            ++ "; }"
                            ++ ".markdown-content em { font-style: italic; }"
                            ++ ".markdown-content blockquote { border-left: 3px solid "
                            ++ UI.colors.border
                            ++ "; padding-left: 0.75rem; margin: 0 0 1rem 0; color: "
                            ++ UI.colors.textMuted
                            ++ "; }"
                            ++ ".markdown-content hr { border: 0; border-top: 1px solid "
                            ++ UI.colors.border
                            ++ "; margin: 1rem 0; }"
                            ++ ".markdown-content wc-mermaid { display: block; margin: 1rem 0; }"
                            ++ "wc-mermaid svg .nodes rect { fill: "
                            ++ UI.colors.bgSecondary
                            ++ " !important; stroke: "
                            ++ UI.colors.textPrimary
                            ++ " !important; stroke-width: 2px !important; }"
                            ++ "wc-mermaid svg .nodes .label { fill: "
                            ++ UI.colors.textPrimary
                            ++ " !important; color: "
                            ++ UI.colors.textPrimary
                            ++ " !important; }"
                            ++ "wc-mermaid svg .edgePath path { stroke: "
                            ++ UI.colors.textPrimary
                            ++ " !important; stroke-width: 2px !important; }"
                            ++ "wc-mermaid svg .edgeLabel { fill: "
                            ++ UI.colors.textPrimary
                            ++ " !important; color: "
                            ++ UI.colors.textPrimary
                            ++ " !important; }"
                            ++ "wc-mermaid svg .edgeLabel rect { fill: "
                            ++ UI.colors.bgPrimary
                            ++ " !important; }"
                            ++ "wc-mermaid svg .cluster rect { fill: "
                            ++ UI.colors.bgSecondary
                            ++ " !important; stroke: "
                            ++ UI.colors.border
                            ++ " !important; }"
                            ++ "wc-mermaid svg text { fill: "
                            ++ UI.colors.textPrimary
                            ++ " !important; }"
                            ++ "wc-mermaid svg .actor { fill: "
                            ++ UI.colors.bgSecondary
                            ++ " !important; stroke: "
                            ++ UI.colors.textPrimary
                            ++ " !important; }"
                            ++ "wc-mermaid svg .actor-line { stroke: "
                            ++ UI.colors.textPrimary
                            ++ " !important; }"
                            ++ "wc-mermaid svg .messageLine0, wc-mermaid svg .messageLine1 { stroke: "
                            ++ UI.colors.textPrimary
                            ++ " !important; }"
                        )
                    ]
                    :: rendered
                )

        Err _ ->
            div
                [ style "white-space" "pre-wrap"
                , style "color" UI.colors.textSecondary
                ]
                [ text markdownString ]


mermaidRenderer : Markdown.Renderer.Renderer (Html msg)
mermaidRenderer =
    let
        defaultRenderer =
            Markdown.Renderer.defaultHtmlRenderer
    in
    { defaultRenderer
        | codeBlock =
            \{ body, language } ->
                case language of
                    Just "mermaid" ->
                        Html.node "wc-mermaid" [ attribute "chart" body ] []

                    _ ->
                        pre [] [ code [] [ text body ] ]
    }


