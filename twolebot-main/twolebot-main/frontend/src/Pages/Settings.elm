module Pages.Settings exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as D
import Types exposing (..)


view :
    RemoteData Settings
    -> Bool
    -> (Bool -> msg)
    -> (Bool -> msg)
    -> (Bool -> msg)
    -> (String -> msg)
    -> (String -> msg)
    -> msg
    -> (String -> msg)
    -> msg
    -> (String -> msg)
    -> msg
    -> (String -> msg)
    -> msg
    -> Maybe ApiKeysData
    -> Bool
    -> Maybe String
    -> String
    -> String
    -> (String -> msg)
    -> (String -> msg)
    -> msg
    -> msg
    -> String
    -> (String -> msg)
    -> msg
    -> msg
    -> Maybe Bool
    -> Maybe String
    -> Html msg
view settingsData isSaving onToggleToolMessages onToggleThinkingMessages onToggleToolResults onChangeChatHarness onChangeClaudeModel onSaveClaudeModel onChangeDevRolePrompt onSaveDevRolePrompt onChangeHardenRolePrompt onSaveHardenRolePrompt onChangePmRolePrompt onSavePmRolePrompt apiKeys apiKeysSaving apiKeysError telegramEdit geminiEdit onTelegramChange onGeminiChange onSaveKeys onRefresh allowedUsernameInput onAllowedUsernameChange onSaveAllowedUsername onClearAllowedUsername hasUserContacted botName =
    case settingsData of
        NotAsked ->
            viewLoading

        Loading ->
            viewLoading

        Failure err ->
            viewError err onRefresh

        Success settings ->
            viewSettings settings isSaving onToggleToolMessages onToggleThinkingMessages onToggleToolResults onChangeChatHarness onChangeClaudeModel onSaveClaudeModel onChangeDevRolePrompt onSaveDevRolePrompt onChangeHardenRolePrompt onSaveHardenRolePrompt onChangePmRolePrompt onSavePmRolePrompt apiKeys apiKeysSaving apiKeysError telegramEdit geminiEdit onTelegramChange onGeminiChange onSaveKeys allowedUsernameInput onAllowedUsernameChange onSaveAllowedUsername onClearAllowedUsername hasUserContacted botName


viewLoading : Html msg
viewLoading =
    div [ class "settings-loading" ]
        [ div [ style "text-align" "center", style "padding" "4rem 2rem", style "color" "#8b949e" ]
            [ text "Loading settings..." ]
        ]


viewError : String -> msg -> Html msg
viewError err onRefresh =
    div [ class "settings-error" ]
        [ div [ style "text-align" "center", style "padding" "2rem", style "color" "#f85149" ]
            [ text ("Error: " ++ err)
            , button
                [ onClick onRefresh
                , style "margin-left" "1rem"
                , style "padding" "0.5rem 1rem"
                , style "background-color" "#21262d"
                , style "color" "#e6edf3"
                , style "border" "1px solid #30363d"
                , style "border-radius" "4px"
                , style "cursor" "pointer"
                ]
                [ text "Retry" ]
            ]
        ]


viewSettings : Settings -> Bool -> (Bool -> msg) -> (Bool -> msg) -> (Bool -> msg) -> (String -> msg) -> (String -> msg) -> msg -> (String -> msg) -> msg -> (String -> msg) -> msg -> (String -> msg) -> msg -> Maybe ApiKeysData -> Bool -> Maybe String -> String -> String -> (String -> msg) -> (String -> msg) -> msg -> String -> (String -> msg) -> msg -> msg -> Maybe Bool -> Maybe String -> Html msg
viewSettings settings isSaving onToggleToolMessages onToggleThinkingMessages onToggleToolResults onChangeChatHarness onChangeClaudeModel onSaveClaudeModel onChangeDevRolePrompt onSaveDevRolePrompt onChangeHardenRolePrompt onSaveHardenRolePrompt onChangePmRolePrompt onSavePmRolePrompt apiKeys apiKeysSaving apiKeysError telegramEdit geminiEdit onTelegramChange onGeminiChange onSaveKeys allowedUsernameInput onAllowedUsernameChange onSaveAllowedUsername onClearAllowedUsername hasUserContacted botName =
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
                [ text "SETTINGS" ]
            , p
                [ style "color" "#8b949e"
                , style "font-size" "0.875rem"
                , style "margin" "0"
                ]
                [ text "Configure twolebot settings and API credentials." ]
            ]

        -- API Keys card
        , viewApiKeysCard apiKeys apiKeysSaving apiKeysError telegramEdit geminiEdit onTelegramChange onGeminiChange onSaveKeys

        -- Access Control card
        , viewAccessControlCard settings isSaving allowedUsernameInput onAllowedUsernameChange onSaveAllowedUsername onClearAllowedUsername hasUserContacted botName

        -- Chat execution harness card
        , viewChatExecutionCard settings isSaving onChangeChatHarness onChangeClaudeModel onSaveClaudeModel

        -- Role Prompts card
        , viewRolePromptsCard settings isSaving onChangeDevRolePrompt onSaveDevRolePrompt onChangeHardenRolePrompt onSaveHardenRolePrompt onChangePmRolePrompt onSavePmRolePrompt

        -- Message Display card
        , div
            [ style "background-color" "#11151c"
            , style "border" "1px solid #21262d"
            , style "border-radius" "6px"
            , style "overflow" "hidden"
            , style "margin-top" "1.5rem"
            ]
            [ -- Section header
              div
                [ style "padding" "1rem 1.5rem"
                , style "border-bottom" "1px solid #21262d"
                , style "background-color" "rgba(0, 212, 170, 0.04)"
                ]
                [ h3
                    [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
                    , style "font-size" "0.875rem"
                    , style "font-weight" "600"
                    , style "color" "#00d4aa"
                    , style "margin" "0"
                    , style "letter-spacing" "0.05em"
                    , style "text-transform" "uppercase"
                    ]
                    [ text "Message Display" ]
                ]

            -- Toggle items
            , div [ style "padding" "0.5rem 0" ]
                [ toggleItem
                    "Show tool usage messages"
                    "Display messages like 'Using tool: Read' when Claude uses tools"
                    settings.showToolMessages
                    isSaving
                    onToggleToolMessages
                , toggleItem
                    "Show thinking messages"
                    "Display Claude's internal reasoning process"
                    settings.showThinkingMessages
                    isSaving
                    onToggleThinkingMessages
                , toggleItem
                    "Show tool results"
                    "Display the results returned by tool calls"
                    settings.showToolResults
                    isSaving
                    onToggleToolResults
                ]
            ]

        -- Info box
        , div
            [ style "margin-top" "1.5rem"
            , style "padding" "1rem 1.25rem"
            , style "background-color" "rgba(0, 212, 170, 0.06)"
            , style "border" "1px solid rgba(0, 212, 170, 0.15)"
            , style "border-radius" "4px"
            , style "display" "flex"
            , style "gap" "0.75rem"
            , style "align-items" "flex-start"
            ]
            [ span
                [ style "color" "#00d4aa"
                , style "font-size" "1.25rem"
                , style "line-height" "1"
                ]
                [ text "i" ]
            , div []
                [ p
                    [ style "color" "#9ca3af"
                    , style "font-size" "0.8125rem"
                    , style "margin" "0"
                    , style "line-height" "1.5"
                    ]
                    [ text "These settings control what gets included in Telegram responses. "
                    , text "Changes take effect on the next Claude process (not the current one)."
                    ]
                ]
            ]
        ]


viewAccessControlCard : Settings -> Bool -> String -> (String -> msg) -> msg -> msg -> Maybe Bool -> Maybe String -> Html msg
viewAccessControlCard settings isSaving usernameInput onUsernameChange onSaveUsername onClearUsername hasUserContacted botName =
    div
        [ style "background-color" "#11151c"
        , style "border" "1px solid #21262d"
        , style "border-radius" "6px"
        , style "overflow" "hidden"
        , style "margin-top" "1.5rem"
        ]
        [ -- Section header
          div
            [ style "padding" "1rem 1.5rem"
            , style "border-bottom" "1px solid #21262d"
            , style "background-color" "rgba(0, 212, 170, 0.04)"
            ]
            [ h3
                [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" "#00d4aa"
                , style "margin" "0"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "Access Control" ]
            ]

        -- Content
        , div [ style "padding" "1rem 1.5rem" ]
            [ case settings.allowedUsername of
                Just username ->
                    div []
                        [ div
                            [ style "display" "flex"
                            , style "justify-content" "space-between"
                            , style "align-items" "center"
                            ]
                            [ div []
                                [ div
                                    [ style "color" "#e6edf3"
                                    , style "font-size" "0.9375rem"
                                    , style "font-weight" "500"
                                    , style "margin-bottom" "0.25rem"
                                    ]
                                    [ text "Allowed Telegram User" ]
                                , div
                                    [ style "font-family" "'IBM Plex Mono', monospace"
                                    , style "font-size" "0.8125rem"
                                    , style "color" "#8b949e"
                                    ]
                                    [ text ("Locked to @" ++ username) ]
                                ]
                            , span
                                [ style "padding" "0.25rem 0.5rem"
                                , style "background-color" "rgba(0, 212, 170, 0.15)"
                                , style "color" "#00d4aa"
                                , style "font-family" "'IBM Plex Mono', monospace"
                                , style "font-size" "0.6875rem"
                                , style "font-weight" "600"
                                , style "letter-spacing" "0.05em"
                                , style "border-radius" "3px"
                                ]
                                [ text "LOCKED" ]
                            ]
                        -- Liveness indicator
                        , case hasUserContacted of
                            Just True ->
                                div
                                    [ style "margin-top" "0.75rem"
                                    , style "padding" "0.5rem 0.75rem"
                                    , style "background-color" "rgba(0, 212, 170, 0.08)"
                                    , style "border" "1px solid rgba(0, 212, 170, 0.25)"
                                    , style "border-radius" "4px"
                                    , style "font-size" "0.8125rem"
                                    , style "color" "#00d4aa"
                                    ]
                                    [ text "Connected — user has messaged the bot" ]

                            Just False ->
                                div
                                    [ style "margin-top" "0.75rem"
                                    , style "padding" "0.5rem 0.75rem"
                                    , style "background-color" "rgba(227, 179, 65, 0.08)"
                                    , style "border" "1px solid rgba(227, 179, 65, 0.25)"
                                    , style "border-radius" "4px"
                                    , style "font-size" "0.8125rem"
                                    , style "color" "#e3b341"
                                    ]
                                    [ text "Waiting — this user hasn't messaged the bot yet. "
                                    , case botName of
                                        Just name ->
                                            a
                                                [ href ("https://t.me/" ++ name)
                                                , target "_blank"
                                                , style "color" "#e3b341"
                                                , style "font-weight" "600"
                                                ]
                                                [ text ("Open @" ++ name ++ " on Telegram") ]

                                        Nothing ->
                                            text "Open the bot on Telegram to start a conversation."
                                    ]

                            Nothing ->
                                text ""
                        ]

                Nothing ->
                    div []
                        [ div
                            [ style "display" "flex"
                            , style "justify-content" "space-between"
                            , style "align-items" "center"
                            , style "margin-bottom" "0.75rem"
                            ]
                            [ div
                                [ style "color" "#e6edf3"
                                , style "font-size" "0.9375rem"
                                , style "font-weight" "500"
                                ]
                                [ text "Allowed Telegram User" ]
                            , span
                                [ style "padding" "0.25rem 0.5rem"
                                , style "background-color" "rgba(248, 81, 73, 0.15)"
                                , style "color" "#f85149"
                                , style "font-family" "'IBM Plex Mono', monospace"
                                , style "font-size" "0.6875rem"
                                , style "font-weight" "600"
                                , style "letter-spacing" "0.05em"
                                , style "border-radius" "3px"
                                ]
                                [ text "NOT SET" ]
                            ]
                        , div
                            [ style "color" "#f85149"
                            , style "font-size" "0.8125rem"
                            , style "margin-bottom" "0.75rem"
                            ]
                            [ text "The bot will not respond to any messages until a username is configured." ]
                        , div
                            [ style "display" "flex"
                            , style "gap" "0.5rem"
                            ]
                            [ input
                                [ type_ "text"
                                , placeholder "Telegram username (e.g. johndoe)"
                                , value usernameInput
                                , onInput onUsernameChange
                                , style "flex" "1"
                                , style "padding" "0.625rem 0.875rem"
                                , style "background-color" "#0d1117"
                                , style "border" "1px solid #30363d"
                                , style "border-radius" "4px"
                                , style "color" "#e6edf3"
                                , style "font-family" "'IBM Plex Mono', monospace"
                                , style "font-size" "0.8125rem"
                                ]
                                []
                            , button
                                [ onClick onSaveUsername
                                , disabled (isSaving || String.isEmpty usernameInput)
                                , style "padding" "0.625rem 1.25rem"
                                , style "background-color" "#238636"
                                , style "color" "#fff"
                                , style "border" "none"
                                , style "border-radius" "4px"
                                , style "font-weight" "600"
                                , style "font-size" "0.875rem"
                                , style "cursor"
                                    (if isSaving || String.isEmpty usernameInput then
                                        "not-allowed"
                                     else
                                        "pointer"
                                    )
                                , style "opacity"
                                    (if isSaving || String.isEmpty usernameInput then
                                        "0.5"
                                     else
                                        "1"
                                    )
                                , style "white-space" "nowrap"
                                ]
                                [ text "Lock" ]
                            ]
                        , div
                            [ style "color" "#6e7681"
                            , style "font-size" "0.75rem"
                            , style "margin-top" "0.5rem"
                            ]
                            [ text "Your Telegram account must have a username set. Go to Telegram Settings > Username if you don't have one." ]
                        ]
            ]
        ]


viewRolePromptsCard : Settings -> Bool -> (String -> msg) -> msg -> (String -> msg) -> msg -> (String -> msg) -> msg -> Html msg
viewRolePromptsCard settings isSaving onChangeDevPrompt onSaveDevPrompt onChangeHardenPrompt onSaveHardenPrompt onChangePmPrompt onSavePmPrompt =
    div
        [ style "background-color" "#11151c"
        , style "border" "1px solid #21262d"
        , style "border-radius" "6px"
        , style "overflow" "hidden"
        , style "margin-top" "1.5rem"
        ]
        [ div
            [ style "padding" "1rem 1.5rem"
            , style "border-bottom" "1px solid #21262d"
            , style "background-color" "rgba(0, 212, 170, 0.04)"
            ]
            [ h3
                [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" "#00d4aa"
                , style "margin" "0"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "SDLC Role Prompts" ]
            ]
        , div
            [ style "padding" "1rem 1.5rem"
            , style "opacity" (if isSaving then "0.6" else "1")
            ]
            [ p
                [ style "color" "#8b949e"
                , style "font-size" "0.8125rem"
                , style "margin" "0 0 1.25rem 0"
                ]
                [ text "Role prompt templates for SDLC phases and chat commands (/pm, /dev, /harden). Use $ARGUMENTS for where task-specific instructions go." ]

            -- Dev prompt
            , div []
                [ div
                    [ style "color" "#e6edf3"
                    , style "font-size" "0.9375rem"
                    , style "font-weight" "500"
                    , style "margin-bottom" "0.4rem"
                    ]
                    [ text "Dev Prompt" ]
                , textarea
                    [ value settings.devRolePrompt
                    , onInput onChangeDevPrompt
                    , on "blur" (D.succeed onSaveDevPrompt)
                    , disabled isSaving
                    , placeholder "Dev role prompt template..."
                    , style "width" "100%"
                    , style "min-height" "14rem"
                    , style "padding" "0.625rem 0.875rem"
                    , style "background-color" "#0d1117"
                    , style "border" "1px solid #30363d"
                    , style "border-radius" "4px"
                    , style "color" "#e6edf3"
                    , style "font-family" "'IBM Plex Mono', monospace"
                    , style "font-size" "0.75rem"
                    , style "line-height" "1.5"
                    , style "resize" "vertical"
                    , style "box-sizing" "border-box"
                    ]
                    []
                ]

            -- Harden prompt
            , div [ style "margin-top" "1.5rem" ]
                [ div
                    [ style "color" "#e6edf3"
                    , style "font-size" "0.9375rem"
                    , style "font-weight" "500"
                    , style "margin-bottom" "0.4rem"
                    ]
                    [ text "Harden Prompt" ]
                , textarea
                    [ value settings.hardenRolePrompt
                    , onInput onChangeHardenPrompt
                    , on "blur" (D.succeed onSaveHardenPrompt)
                    , disabled isSaving
                    , placeholder "Harden role prompt template..."
                    , style "width" "100%"
                    , style "min-height" "14rem"
                    , style "padding" "0.625rem 0.875rem"
                    , style "background-color" "#0d1117"
                    , style "border" "1px solid #30363d"
                    , style "border-radius" "4px"
                    , style "color" "#e6edf3"
                    , style "font-family" "'IBM Plex Mono', monospace"
                    , style "font-size" "0.75rem"
                    , style "line-height" "1.5"
                    , style "resize" "vertical"
                    , style "box-sizing" "border-box"
                    ]
                    []
                ]

            -- PM prompt
            , div [ style "margin-top" "1.5rem" ]
                [ div
                    [ style "color" "#e6edf3"
                    , style "font-size" "0.9375rem"
                    , style "font-weight" "500"
                    , style "margin-bottom" "0.4rem"
                    ]
                    [ text "PM Prompt" ]
                , p
                    [ style "color" "#8b949e"
                    , style "font-size" "0.75rem"
                    , style "margin" "0 0 0.5rem 0"
                    ]
                    [ text "Used by the /pm chat command in Telegram and web chat." ]
                , textarea
                    [ value settings.pmRolePrompt
                    , onInput onChangePmPrompt
                    , on "blur" (D.succeed onSavePmPrompt)
                    , disabled isSaving
                    , placeholder "PM role prompt template..."
                    , style "width" "100%"
                    , style "min-height" "14rem"
                    , style "padding" "0.625rem 0.875rem"
                    , style "background-color" "#0d1117"
                    , style "border" "1px solid #30363d"
                    , style "border-radius" "4px"
                    , style "color" "#e6edf3"
                    , style "font-family" "'IBM Plex Mono', monospace"
                    , style "font-size" "0.75rem"
                    , style "line-height" "1.5"
                    , style "resize" "vertical"
                    , style "box-sizing" "border-box"
                    ]
                    []
                ]
            ]
        ]


viewChatExecutionCard : Settings -> Bool -> (String -> msg) -> (String -> msg) -> msg -> Html msg
viewChatExecutionCard settings isSaving onChangeHarness onChangeModel onSaveModel =
    let
        selectedHarness =
            case String.toLower (String.trim settings.chatHarness) of
                "codex" ->
                    "codex"

                "echo" ->
                    "echo"

                _ ->
                    "claude"
    in
    div
        [ style "background-color" "#11151c"
        , style "border" "1px solid #21262d"
        , style "border-radius" "6px"
        , style "overflow" "hidden"
        , style "margin-top" "1.5rem"
        ]
        [ div
            [ style "padding" "1rem 1.5rem"
            , style "border-bottom" "1px solid #21262d"
            , style "background-color" "rgba(0, 212, 170, 0.04)"
            ]
            [ h3
                [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" "#00d4aa"
                , style "margin" "0"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "Chat Execution" ]
            ]
        , div
            [ style "padding" "1rem 1.5rem"
            , style "opacity" (if isSaving then "0.6" else "1")
            ]
            [ div
                [ style "color" "#e6edf3"
                , style "font-size" "0.9375rem"
                , style "font-weight" "500"
                , style "margin-bottom" "0.4rem"
                ]
                [ text "Harness" ]
            , p
                [ style "color" "#8b949e"
                , style "font-size" "0.8125rem"
                , style "margin" "0 0 0.75rem 0"
                ]
                [ text "Select which CLI handles Telegram/Web chat prompts." ]
            , select
                [ value selectedHarness
                , onInput onChangeHarness
                , disabled isSaving
                , style "width" "100%"
                , style "max-width" "280px"
                , style "padding" "0.625rem 0.875rem"
                , style "background-color" "#0d1117"
                , style "border" "1px solid #30363d"
                , style "border-radius" "4px"
                , style "color" "#e6edf3"
                , style "font-family" "'IBM Plex Mono', monospace"
                , style "font-size" "0.8125rem"
                ]
                [ option [ value "claude" ] [ text "Claude CLI (default)" ]
                , option [ value "codex" ] [ text "Codex CLI" ]
                , option [ value "echo" ] [ text "Echo (test harness)" ]
                ]
            , div [ style "margin-top" "1.5rem" ]
                [ div
                    [ style "color" "#e6edf3"
                    , style "font-size" "0.9375rem"
                    , style "font-weight" "500"
                    , style "margin-bottom" "0.4rem"
                    ]
                    [ text "Claude Model" ]
                , p
                    [ style "color" "#8b949e"
                    , style "font-size" "0.8125rem"
                    , style "margin" "0 0 0.75rem 0"
                    ]
                    [ text "Model passed to Claude CLI via --model. Per-instance (stored locally, not global)." ]
                , input
                    [ value settings.claudeModel
                    , onInput onChangeModel
                    , on "blur" (D.succeed onSaveModel)
                    , disabled isSaving
                    , Html.Attributes.list "claude-models"
                    , placeholder "e.g. claude-opus-4-6"
                    , style "width" "100%"
                    , style "max-width" "380px"
                    , style "padding" "0.625rem 0.875rem"
                    , style "background-color" "#0d1117"
                    , style "border" "1px solid #30363d"
                    , style "border-radius" "4px"
                    , style "color" "#e6edf3"
                    , style "font-family" "'IBM Plex Mono', monospace"
                    , style "font-size" "0.8125rem"
                    ]
                    []
                , Html.node "datalist"
                    [ id "claude-models" ]
                    [ option [ value "claude-opus-4-6" ] []
                    , option [ value "claude-sonnet-4-20250514" ] []
                    , option [ value "claude-sonnet-4-5-20250514" ] []
                    , option [ value "claude-haiku-3-5-20241022" ] []
                    ]
                ]
            ]
        ]


viewApiKeysCard : Maybe ApiKeysData -> Bool -> Maybe String -> String -> String -> (String -> msg) -> (String -> msg) -> msg -> Html msg
viewApiKeysCard maybeApiKeys isSaving maybeError telegramEdit geminiEdit onTelegramChange onGeminiChange onSaveKeys =
    div
        [ style "background-color" "#11151c"
        , style "border" "1px solid #21262d"
        , style "border-radius" "6px"
        , style "overflow" "hidden"
        ]
        [ -- Section header
          div
            [ style "padding" "1rem 1.5rem"
            , style "border-bottom" "1px solid #21262d"
            , style "background-color" "rgba(0, 212, 170, 0.04)"
            ]
            [ h3
                [ style "font-family" "'IBM Plex Sans Condensed', sans-serif"
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" "#00d4aa"
                , style "margin" "0"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "API Keys" ]
            ]

        -- Content
        , div [ style "padding" "1rem 1.5rem" ]
            [ case maybeApiKeys of
                Nothing ->
                    div [ style "color" "#8b949e", style "text-align" "center", style "padding" "1rem" ]
                        [ text "Loading API key status..." ]

                Just apiKeys ->
                    div []
                        [ -- Claude Code status
                          viewClaudeCodeRow apiKeys.claudeCodeStatus

                        -- Telegram token
                        , viewApiKeyRow
                            "Telegram Bot Token"
                            apiKeys.hasTelegramToken
                            apiKeys.telegramTokenMasked
                            apiKeys.telegramStatus
                            telegramEdit
                            "Enter new Telegram token..."
                            onTelegramChange

                        -- Gemini API Key
                        , viewApiKeyRow
                            "Gemini API Key"
                            apiKeys.hasGeminiKey
                            apiKeys.geminiKeyMasked
                            apiKeys.geminiStatus
                            geminiEdit
                            "Enter new Gemini key..."
                            onGeminiChange

                        -- Error message
                        , case maybeError of
                            Just err ->
                                div
                                    [ style "margin-top" "1rem"
                                    , style "padding" "0.75rem 1rem"
                                    , style "background-color" "rgba(248, 81, 73, 0.1)"
                                    , style "border" "1px solid #f85149"
                                    , style "border-radius" "4px"
                                    , style "color" "#f85149"
                                    , style "font-size" "0.8125rem"
                                    ]
                                    [ text err ]

                            Nothing ->
                                text ""

                        -- Save button
                        , if not (String.isEmpty telegramEdit) || not (String.isEmpty geminiEdit) then
                            div [ style "margin-top" "1rem", style "text-align" "right" ]
                                [ button
                                    [ onClick onSaveKeys
                                    , disabled isSaving
                                    , style "padding" "0.625rem 1.25rem"
                                    , style "background-color" "#238636"
                                    , style "color" "#fff"
                                    , style "border" "none"
                                    , style "border-radius" "4px"
                                    , style "font-weight" "600"
                                    , style "font-size" "0.875rem"
                                    , style "cursor" (if isSaving then "wait" else "pointer")
                                    , style "opacity" (if isSaving then "0.7" else "1")
                                    ]
                                    [ text (if isSaving then "Saving..." else "Save Changes") ]
                                ]
                          else
                            text ""
                        ]
            ]
        ]


viewApiKeyRow : String -> Bool -> Maybe String -> Maybe ApiKeyStatus -> String -> String -> (String -> msg) -> Html msg
viewApiKeyRow label hasKey maskedKey maybeStatus editValue placeholderText onChange =
    div
        [ style "padding" "1rem 0"
        , style "border-bottom" "1px solid #21262d"
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-bottom" "0.5rem"
            ]
            [ span
                [ style "color" "#e6edf3"
                , style "font-weight" "500"
                , style "font-size" "0.9375rem"
                ]
                [ text label ]
            , viewStatusBadge hasKey maybeStatus
            ]

        -- Current value (masked)
        , case maskedKey of
            Just masked ->
                div
                    [ style "font-family" "'IBM Plex Mono', monospace"
                    , style "font-size" "0.8125rem"
                    , style "color" "#8b949e"
                    , style "margin-bottom" "0.75rem"
                    ]
                    [ text ("Current: " ++ masked) ]

            Nothing ->
                div
                    [ style "font-size" "0.8125rem"
                    , style "color" "#6e7681"
                    , style "margin-bottom" "0.75rem"
                    ]
                    [ text "Not configured" ]

        -- Status info
        , case maybeStatus of
            Just status ->
                case ( status.valid, status.info, status.error ) of
                    ( True, Just info, _ ) ->
                        div
                            [ style "font-size" "0.75rem"
                            , style "color" "#00d4aa"
                            , style "margin-bottom" "0.75rem"
                            ]
                            [ text info ]

                    ( False, _, Just err ) ->
                        div
                            [ style "font-size" "0.75rem"
                            , style "color" "#f85149"
                            , style "margin-bottom" "0.75rem"
                            ]
                            [ text err ]

                    _ ->
                        text ""

            Nothing ->
                text ""

        -- Input for new value
        , input
            [ type_ "password"
            , placeholder placeholderText
            , value editValue
            , onInput onChange
            , style "width" "100%"
            , style "padding" "0.625rem 0.875rem"
            , style "background-color" "#0d1117"
            , style "border" "1px solid #30363d"
            , style "border-radius" "4px"
            , style "color" "#e6edf3"
            , style "font-family" "'IBM Plex Mono', monospace"
            , style "font-size" "0.8125rem"
            ]
            []
        ]


viewClaudeCodeRow : Maybe ClaudeCodeStatus -> Html msg
viewClaudeCodeRow maybeStatus =
    div
        [ style "padding" "1rem 0"
        , style "border-bottom" "1px solid #21262d"
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-bottom" "0.5rem"
            ]
            [ span
                [ style "color" "#e6edf3"
                , style "font-weight" "500"
                , style "font-size" "0.9375rem"
                ]
                [ text "Claude Code" ]
            , case maybeStatus of
                Just status ->
                    span
                        [ style "padding" "0.25rem 0.5rem"
                        , style "background-color" "rgba(0, 212, 170, 0.15)"
                        , style "color" "#00d4aa"
                        , style "font-family" "'IBM Plex Mono', monospace"
                        , style "font-size" "0.6875rem"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.05em"
                        , style "border-radius" "3px"
                        ]
                        [ text (String.toUpper status.authMode) ]

                Nothing ->
                    span
                        [ style "padding" "0.25rem 0.5rem"
                        , style "background-color" "rgba(139, 148, 158, 0.15)"
                        , style "color" "#8b949e"
                        , style "font-family" "'IBM Plex Mono', monospace"
                        , style "font-size" "0.6875rem"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.05em"
                        , style "border-radius" "3px"
                        ]
                        [ text "NOT FOUND" ]
            ]
        , case maybeStatus of
            Just status ->
                div []
                    [ case status.accountEmail of
                        Just email ->
                            div
                                [ style "font-size" "0.8125rem"
                                , style "color" "#8b949e"
                                , style "margin-bottom" "0.25rem"
                                ]
                                [ text email ]

                        Nothing ->
                            text ""
                    , case status.organization of
                        Just org ->
                            div
                                [ style "font-size" "0.75rem"
                                , style "color" "#00d4aa"
                                ]
                                [ text org ]

                        Nothing ->
                            text ""
                    ]

            Nothing ->
                div
                    [ style "font-size" "0.8125rem"
                    , style "color" "#6e7681"
                    ]
                    [ text "Claude CLI not configured or ~/.claude.json not found" ]
        ]


viewStatusBadge : Bool -> Maybe ApiKeyStatus -> Html msg
viewStatusBadge hasKey maybeStatus =
    let
        ( bgColor, textColor, labelText ) =
            if not hasKey then
                ( "rgba(248, 81, 73, 0.15)", "#f85149", "NOT SET" )
            else
                case maybeStatus of
                    Just status ->
                        if status.valid then
                            ( "rgba(0, 212, 170, 0.15)", "#00d4aa", "VALID" )
                        else
                            ( "rgba(248, 81, 73, 0.15)", "#f85149", "INVALID" )

                    Nothing ->
                        ( "rgba(139, 148, 158, 0.15)", "#8b949e", "CHECKING..." )
    in
    span
        [ style "padding" "0.25rem 0.5rem"
        , style "background-color" bgColor
        , style "color" textColor
        , style "font-family" "'IBM Plex Mono', monospace"
        , style "font-size" "0.6875rem"
        , style "font-weight" "600"
        , style "letter-spacing" "0.05em"
        , style "border-radius" "3px"
        ]
        [ text labelText ]


toggleItem : String -> String -> Bool -> Bool -> (Bool -> msg) -> Html msg
toggleItem label description isOn isSaving onToggle =
    div
        [ style "display" "flex"
        , style "justify-content" "space-between"
        , style "align-items" "center"
        , style "padding" "1rem 1.5rem"
        , style "border-bottom" "1px solid #21262d"
        , style "opacity" (if isSaving then "0.6" else "1")
        ]
        [ div [ style "flex" "1", style "margin-right" "1rem" ]
            [ div
                [ style "color" "#e6edf3"
                , style "font-size" "0.9375rem"
                , style "font-weight" "500"
                , style "margin-bottom" "0.25rem"
                ]
                [ text label ]
            , div
                [ style "color" "#8b949e"
                , style "font-size" "0.8125rem"
                ]
                [ text description ]
            ]
        , toggleSwitch isOn isSaving onToggle
        ]


toggleSwitch : Bool -> Bool -> (Bool -> msg) -> Html msg
toggleSwitch isOn isSaving onToggle =
    button
        [ onClick (onToggle (not isOn))
        , disabled isSaving
        , style "position" "relative"
        , style "width" "48px"
        , style "height" "26px"
        , style "background-color" (if isOn then "#00d4aa" else "#30363d")
        , style "border" "none"
        , style "border-radius" "13px"
        , style "cursor" (if isSaving then "wait" else "pointer")
        , style "transition" "background-color 0.2s ease"
        , style "flex-shrink" "0"
        ]
        [ span
            [ style "position" "absolute"
            , style "top" "3px"
            , style "left" (if isOn then "25px" else "3px")
            , style "width" "20px"
            , style "height" "20px"
            , style "background-color" "#ffffff"
            , style "border-radius" "50%"
            , style "transition" "left 0.2s ease"
            , style "box-shadow" "0 1px 3px rgba(0,0,0,0.3)"
            ]
            []
        ]
