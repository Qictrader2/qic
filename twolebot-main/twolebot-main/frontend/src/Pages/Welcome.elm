module Pages.Welcome exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)


view : Html msg
view =
    div []
        [ -- Page header
          div
            [ style "margin-bottom" "2rem" ]
            [ h2
                [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
                , style "font-size" "1.5rem"
                , style "font-weight" "600"
                , style "color" "#e6edf3"
                , style "margin" "0 0 0.5rem 0"
                , style "letter-spacing" "0.02em"
                ]
                [ text "WELCOME TO TWOLEBOT" ]
            , p
                [ style "color" "#8b949e"
                , style "font-size" "0.875rem"
                , style "margin" "0"
                ]
                [ text "Claude on Telegram - your AI assistant is ready to help." ]
            ]

        -- Quick Start section
        , viewSection "Quick Start"
            [ viewExampleCard
                "Say Hello"
                "Just send any message to your bot on Telegram. Claude will respond conversationally."
                [ "Hi there!"
                , "What can you help me with?"
                ]
            , viewExampleCard
                "Ask Questions"
                "Get help with coding, writing, analysis, math, and more."
                [ "Explain how async/await works in JavaScript"
                , "What's the capital of Mongolia?"
                ]
            ]

        -- Voice & Media section
        , viewSection "Voice & Media"
            [ viewExampleCard
                "Voice Messages"
                "Send voice notes - they're automatically transcribed and Claude responds to the content."
                [ "(Send a voice message asking a question)"
                , "(Describe a problem verbally)"
                ]
            , viewExampleCard
                "Images"
                "Share photos for analysis, OCR, or visual questions."
                [ "(Send a screenshot of an error)"
                , "(Share a photo and ask what's in it)"
                ]
            ]

        -- Advanced Features section
        , viewSection "Scheduled Tasks"
            [ viewExampleCard
                "One-Shot Reminders"
                "Schedule Claude to send you something later."
                [ "Remind me in 2 hours to check the deployment"
                , "Send me a motivational quote at 8am tomorrow"
                ]
            , viewExampleCard
                "Recurring Jobs"
                "Set up daily, weekly, or custom schedules via MCP."
                [ "Every morning at 9am, summarize tech news"
                , "Weekly on Monday, review my calendar"
                ]
            ]

        -- Clawdbot-Inspired section
        , viewSection "Try These Now"
            [ viewPromptCard "Explain this like I'm 5: [paste any complex topic]"
            , viewPromptCard "Write a haiku about my current mood"
            , viewPromptCard "Give me 3 creative names for a pet hamster"
            , viewPromptCard "What's a fun fact I probably don't know?"
            , viewPromptCard "Help me write a polite email declining a meeting"
            , viewPromptCard "Suggest a quick 15-minute productivity technique"
            ]

        -- Tips section
        , viewTipsSection
        ]


viewSection : String -> List (Html msg) -> Html msg
viewSection title content =
    div
        [ style "margin-bottom" "2rem" ]
        [ h3
            [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
            , style "font-size" "0.875rem"
            , style "font-weight" "600"
            , style "color" "#00d4aa"
            , style "margin" "0 0 1rem 0"
            , style "letter-spacing" "0.05em"
            , style "text-transform" "uppercase"
            ]
            [ text title ]
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(280px, 1fr))"
            , style "gap" "1rem"
            ]
            content
        ]


viewExampleCard : String -> String -> List String -> Html msg
viewExampleCard title description examples =
    div
        [ style "background-color" "#11151c"
        , style "border" "1px solid #21262d"
        , style "border-radius" "6px"
        , style "padding" "1.25rem"
        ]
        [ h4
            [ style "color" "#e6edf3"
            , style "font-size" "1rem"
            , style "font-weight" "600"
            , style "margin" "0 0 0.5rem 0"
            ]
            [ text title ]
        , p
            [ style "color" "#8b949e"
            , style "font-size" "0.8125rem"
            , style "margin" "0 0 1rem 0"
            , style "line-height" "1.5"
            ]
            [ text description ]
        , div
            [ style "display" "flex"
            , style "flex-direction" "column"
            , style "gap" "0.5rem"
            ]
            (List.map viewExampleItem examples)
        ]


viewExampleItem : String -> Html msg
viewExampleItem example =
    div
        [ style "background-color" "#0d1117"
        , style "border" "1px solid #30363d"
        , style "border-radius" "4px"
        , style "padding" "0.5rem 0.75rem"
        , style "font-family" "'IBM Plex Mono', monospace"
        , style "font-size" "0.75rem"
        , style "color" "#7ee787"
        ]
        [ text example ]


viewPromptCard : String -> Html msg
viewPromptCard prompt =
    div
        [ style "background-color" "#11151c"
        , style "border" "1px solid #21262d"
        , style "border-radius" "6px"
        , style "padding" "1rem"
        , style "font-family" "'IBM Plex Mono', monospace"
        , style "font-size" "0.8125rem"
        , style "color" "#e6edf3"
        , style "line-height" "1.5"
        , style "cursor" "default"
        , style "transition" "border-color 0.2s ease"
        ]
        [ span [ style "color" "#00d4aa", style "margin-right" "0.5rem" ] [ text ">" ]
        , text prompt
        ]


viewTipsSection : Html msg
viewTipsSection =
    div
        [ style "margin-top" "2rem"
        , style "padding" "1.25rem"
        , style "background-color" "rgba(0, 212, 170, 0.06)"
        , style "border" "1px solid rgba(0, 212, 170, 0.15)"
        , style "border-radius" "6px"
        ]
        [ h4
            [ style "color" "#00d4aa"
            , style "font-size" "0.875rem"
            , style "font-weight" "600"
            , style "margin" "0 0 1rem 0"
            , style "letter-spacing" "0.02em"
            ]
            [ text "Tips" ]
        , ul
            [ style "margin" "0"
            , style "padding" "0 0 0 1.25rem"
            , style "color" "#9ca3af"
            , style "font-size" "0.8125rem"
            , style "line-height" "1.8"
            ]
            [ li [] [ text "Send ", strong [ style "color" "#e6edf3" ] [ text "/clear" ], text " to reset conversation context and start fresh" ]
            , li [] [ text "Claude remembers context within a conversation - refer back to earlier messages" ]
            , li [] [ text "For long tasks, Claude may send multiple messages as it works" ]
            , li [] [ text "Voice messages work great for quick questions while on the go" ]
            , li [] [ text "Check the ", strong [ style "color" "#e6edf3" ] [ text "Dashboard" ], text " to see active prompts and responses in real-time" ]
            ]
        ]
