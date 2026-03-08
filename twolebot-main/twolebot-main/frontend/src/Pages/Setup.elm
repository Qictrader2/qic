module Pages.Setup exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput, onSubmit)
import Types exposing (..)
import UI


view :
    SetupStatus
    -> String
    -> String
    -> String
    -> SetupMsgs msg
    -> Html msg
view status telegramToken geminiKey usernameInput msgs =
    div [ class "setup-page" ]
        [ viewHeader status
        , div [ class "setup-cards" ]
            [ cardTelegram status telegramToken msgs
            , cardUsername status usernameInput msgs
            , cardClaude status msgs
            , cardThreading status msgs
            , cardGemini status geminiKey msgs
            ]
        , viewDashboardButton status msgs.goToDashboard
        , viewStyles
        ]


-- ── Header ────────────────────────────────────────────────────────────────────


isAllComplete : SetupStatus -> Bool
isAllComplete status =
    status.hasTelegramToken
        && status.hasAllowedUsername
        && status.hasClaudeCli
        && status.claudeAuthenticated
        && status.hasThreadingEnabled


viewHeader : SetupStatus -> Html msg
viewHeader status =
    let
        required =
            [ status.hasTelegramToken
            , status.hasAllowedUsername
            , status.hasClaudeCli && status.claudeAuthenticated
            , status.hasThreadingEnabled
            ]
        done = List.length (List.filter identity required)
        total = List.length required
    in
    div [ class "setup-header" ]
        [ h1 [] [ text "twolebot" ]
        , p [ class "tagline" ] [ text "Claude on Telegram" ]
        , p [ class "setup-progress-text" ]
            [ text (String.fromInt done ++ " / " ++ String.fromInt total ++ " required complete") ]
        ]


-- ── Dashboard button ──────────────────────────────────────────────────────────


viewDashboardButton : SetupStatus -> msg -> Html msg
viewDashboardButton status goMsg =
    div [ class "dashboard-row" ]
        [ if isAllComplete status then
            button
                [ class "dashboard-button ready"
                , onClick goMsg
                ]
                [ text "Go to Dashboard →" ]
          else
            div [ class "dashboard-button-disabled" ]
                [ text "Complete the required steps above to continue" ]
        ]


-- ── Card helpers ──────────────────────────────────────────────────────────────


cardShell : Bool -> List (Html msg) -> Html msg
cardShell complete children =
    div [ class "setup-card", classList [ ( "card-complete", complete ) ] ]
        children


cardTitle : Bool -> Bool -> String -> String -> Html msg
cardTitle complete required label badge =
    div [ class "card-title-row" ]
        [ div [ class "card-status-icon", classList [ ( "icon-complete", complete ) ] ]
            [ text (if complete then "✓" else "·") ]
        , h2 [ class "card-title" ] [ text label ]
        , if not required then
            span [ class "optional-badge" ] [ text "OPTIONAL" ]
          else if complete then
            span [ class "badge-done" ] [ text badge ]
          else
            text ""
        ]


-- ── Telegram token card ───────────────────────────────────────────────────────


cardTelegram : SetupStatus -> String -> SetupMsgs msg -> Html msg
cardTelegram status token msgs =
    cardShell status.hasTelegramToken
        [ cardTitle status.hasTelegramToken True "Telegram Bot" ""
        , if status.hasTelegramToken then
            div [ class "card-verified" ]
                [ text "Connected"
                , case status.botName of
                    Just name -> span [ class "verified-detail" ] [ text (" · @" ++ name) ]
                    Nothing -> text ""
                ]
          else
            text ""
        , div [ class "card-instructions" ]
            [ p []
                [ text "Create a bot with "
                , a [ href "https://t.me/BotFather?start=newbot", target "_blank" ] [ text "BotFather" ]
                , text ", then paste the token here."
                ]
            ]
        , Html.form [ onSubmit msgs.submitTelegram, class "card-form" ]
            [ input
                [ type_ "text"
                , placeholder "Bot token (e.g. 123456789:AAG…)"
                , value token
                , onInput msgs.telegramInput
                , class "card-input"
                , autocomplete False
                , spellcheck False
                ]
                []
            , button
                [ type_ "submit"
                , class "card-button"
                , disabled (String.isEmpty token)
                ]
                [ text (if status.hasTelegramToken then "Update" else "Verify & Save") ]
            ]
        , case status.telegramError of
            Just err -> div [ class "card-error" ] [ text err ]
            Nothing -> text ""
        ]


-- ── Username card ─────────────────────────────────────────────────────────────


cardUsername : SetupStatus -> String -> SetupMsgs msg -> Html msg
cardUsername status usernameInput msgs =
    cardShell status.hasAllowedUsername
        [ cardTitle status.hasAllowedUsername True "Your Telegram Username" ""
        , case status.allowedUsernameValue of
            Just u ->
                div [ class "card-verified" ]
                    [ text "Configured"
                    , span [ class "verified-detail" ] [ text (" · @" ++ u) ]
                    ]
            Nothing ->
                if status.hasAllowedUsername then
                    div [ class "card-verified" ] [ text "Configured" ]
                else
                    text ""
        , div [ class "card-instructions" ]
            [ p [] [ text "Only messages from this username will be processed — required for security." ]
            ]
        , Html.form [ onSubmit msgs.submitUsername, class "card-form" ]
            [ input
                [ type_ "text"
                , placeholder "Username (e.g. johndoe)"
                , value usernameInput
                , onInput msgs.usernameInput
                , class "card-input"
                , autocomplete False
                , spellcheck False
                ]
                []
            , button
                [ type_ "submit"
                , class "card-button"
                , disabled (String.isEmpty (String.trim usernameInput))
                ]
                [ text (if status.hasAllowedUsername then "Update" else "Save") ]
            ]
        , case status.allowedUsernameError of
            Just err -> div [ class "card-error" ] [ text err ]
            Nothing -> text ""
        ]


-- ── Claude card ───────────────────────────────────────────────────────────────


cardClaude : SetupStatus -> SetupMsgs msg -> Html msg
cardClaude status msgs =
    let
        claudeDone = status.hasClaudeCli && status.claudeAuthenticated
    in
    cardShell claudeDone
        [ cardTitle claudeDone True "Claude CLI" ""
        , div [ class "card-instructions" ]
            [ p []
                [ text "Requires a "
                , strong [] [ text "Max or Pro subscription" ]
                , text ". "
                , a [ href "https://claude.ai/settings/billing", target "_blank" ] [ text "Check billing →" ]
                ]
            ]
        , if status.hasClaudeCli then
            div []
                [ div [ class "card-verified" ]
                    [ text "Installed"
                    , case status.claudeCliVersion of
                        Just v -> span [ class "verified-detail" ] [ text (" · " ++ v) ]
                        Nothing -> text ""
                    ]
                , if status.claudeNeedsUpdate then
                    div [ class "card-warn" ]
                        [ text "Update available"
                        , case status.claudeLatestVersion of
                            Just v -> span [ class "verified-detail" ] [ text (" · " ++ v) ]
                            Nothing -> text ""
                        , if status.claudeUpdating then
                            span [ class "card-inline-loading" ] [ text " Updating…" ]
                          else
                            button [ class "card-button-inline", onClick msgs.updateClaude ] [ text "Update" ]
                        , case status.claudeUpdateError of
                            Just err -> div [ class "card-error" ] [ text err ]
                            Nothing -> text ""
                        ]
                  else
                    text ""
                , if status.claudeAuthenticated then
                    div [ class "card-verified", style "margin-top" "0.5rem" ]
                        [ text "Authenticated"
                        , case status.claudeAuthMode of
                            Just "oauth" ->
                                span [ class "verified-detail" ]
                                    [ text " · OAuth"
                                    , case status.claudeAccountEmail of
                                        Just email -> text (" — " ++ email)
                                        Nothing -> text ""
                                    ]
                            Just "api_key" -> span [ class "verified-detail" ] [ text " · API key" ]
                            _ -> text ""
                        ]
                  else if status.claudeAuthChecking then
                    div [ class "card-loading" ] [ text "Checking auth…" ]
                  else
                    div []
                        [ div [ class "card-error" ] [ text "Not authenticated — run claude to log in" ]
                        , viewAuthInstructions
                        , button [ class "card-button", style "margin-top" "0.75rem", onClick msgs.checkClaudeAuth ]
                            [ text "Check again" ]
                        ]
                , if status.claudeAuthenticated then
                    div [ class "card-test-row" ]
                        [ if status.claudeTesting then
                            span [ class "card-loading" ] [ text "Testing…" ]
                          else
                            button [ class "card-button-secondary", onClick msgs.testClaude ] [ text "Test Claude" ]
                        , case status.claudeTestResult of
                            Just True ->
                                span [ class "test-pass" ]
                                    [ text ("✓ " ++ Maybe.withDefault "OK" status.claudeTestOutput) ]
                            Just False ->
                                span [ class "test-fail" ]
                                    [ text (Maybe.withDefault "Test failed" status.claudeTestError) ]
                            Nothing -> text ""
                        ]
                  else
                    text ""
                ]
          else if status.claudeInstalling then
            div [ class "card-loading" ] [ text "Installing via npm… this may take a minute" ]
          else
            div []
                [ case status.claudeInstallError of
                    Just err -> div [ class "card-error" ] [ text err ]
                    Nothing -> text ""
                , div [ class "card-install-row" ]
                    [ button [ class "card-button", onClick msgs.installClaude ] [ text "Install Claude CLI" ]
                    , span [ class "card-hint" ] [ text "requires Node.js/npm" ]
                    ]
                , div [ class "card-manual" ]
                    [ p [ class "card-hint" ] [ text "Or manually:" ]
                    , pre [ class "code-block" ]
                        [ code [] [ text "npm install -g @anthropic-ai/claude-code" ] ]
                    , button [ class "card-button-secondary", style "margin-top" "0.5rem", onClick msgs.checkClaudeAuth ]
                        [ text "I've installed it — check" ]
                    ]
                ]
        ]


viewAuthInstructions : Html msg
viewAuthInstructions =
    div [ class "auth-options", style "margin-top" "0.75rem" ]
        [ div [ class "auth-option" ]
            [ strong [] [ text "Desktop" ]
            , pre [ class "code-block" ] [ code [] [ text "claude" ] ]
            ]
        , div [ class "or-divider" ] [ text "or" ]
        , div [ class "auth-option" ]
            [ strong [] [ text "Headless / VPS" ]
            , pre [ class "code-block" ] [ code [] [ text "claude setup-token" ] ]
            , p [ class "card-hint" ]
                [ text "Get a token from "
                , a [ href "https://claude.ai/settings/tokens", target "_blank" ] [ text "claude.ai/settings/tokens" ]
                ]
            ]
        ]


-- ── Threading card ────────────────────────────────────────────────────────────


cardThreading : SetupStatus -> SetupMsgs msg -> Html msg
cardThreading status msgs =
    cardShell status.hasThreadingEnabled
        [ cardTitle status.hasThreadingEnabled True "Topics / Threading" ""
        , if status.hasThreadingEnabled then
            div [ class "card-verified" ] [ text "Enabled" ]
          else
            text ""
        , div [ class "card-instructions" ]
            [ p [] [ text "Lets you run multiple conversations with your bot simultaneously." ]
            , ol [ class "setup-ol" ]
                [ li [] [ text "Open @BotFather on Telegram" ]
                , li []
                    [ text "Click "
                    , strong [] [ text "Open" ]
                    , text " (bottom-left) to launch the BotFather miniapp"
                    ]
                , li [] [ text "Select your bot" ]
                , li [] [ text "Go to Bot Settings" ]
                , li []
                    [ text "Enable "
                    , strong [] [ text "Threading" ]
                    ]
                ]
            ]
        , case status.threadingError of
            Just err -> div [ class "card-error" ] [ text err ]
            Nothing -> text ""
        , if not status.hasThreadingEnabled then
            button
                [ class "card-button"
                , onClick msgs.checkThreading
                , disabled status.threadingChecking
                ]
                [ text (if status.threadingChecking then "Checking…" else "I've enabled it — verify") ]
          else
            text ""
        ]


-- ── Gemini card ───────────────────────────────────────────────────────────────


cardGemini : SetupStatus -> String -> SetupMsgs msg -> Html msg
cardGemini status key msgs =
    cardShell status.hasGeminiKey
        [ cardTitle status.hasGeminiKey False "Gemini API Key" ""
        , if status.hasGeminiKey then
            div [ class "card-verified" ]
                [ text "Verified"
                , case status.geminiKeyPreview of
                    Just preview -> span [ class "verified-detail" ] [ text (" · " ++ preview) ]
                    Nothing -> text ""
                ]
          else
            text ""
        , div [ class "card-instructions" ]
            [ p [] [ text "Enables voice transcription and image descriptions." ]
            , a [ href "https://aistudio.google.com/apikey", target "_blank", class "card-link" ]
                [ text "Get free API key →" ]
            ]
        , Html.form [ onSubmit msgs.submitGemini, class "card-form" ]
            [ input
                [ type_ "text"
                , placeholder (if status.hasGeminiKey then "Paste new key to update" else "Gemini API key")
                , value key
                , onInput msgs.geminiInput
                , class "card-input"
                , autocomplete False
                , spellcheck False
                ]
                []
            , button
                [ type_ "submit"
                , class "card-button"
                , disabled (String.isEmpty key)
                ]
                [ text (if status.hasGeminiKey then "Update" else "Verify & Save") ]
            ]
        , case status.geminiError of
            Just err -> div [ class "card-error" ] [ text err ]
            Nothing -> text ""
        ]


-- ── Styles ────────────────────────────────────────────────────────────────────


viewStyles : Html msg
viewStyles =
    node "style" []
        [ text """
            .setup-page {
                max-width: 600px;
                margin: 2rem auto;
                padding: 0 1rem 4rem;
                font-family: 'IBM Plex Sans', -apple-system, sans-serif;
                color: #e6edf3;
            }

            .setup-header {
                text-align: center;
                margin-bottom: 2.5rem;
            }

            .setup-header h1 {
                font-family: 'IBM Plex Mono', monospace;
                font-size: 2rem;
                font-weight: 600;
                color: #00d4aa;
                margin: 0 0 0.25rem;
            }

            .tagline {
                color: #8b949e;
                font-size: 0.9rem;
                margin: 0 0 0.5rem;
            }

            .setup-progress-text {
                color: #8b949e;
                font-family: 'IBM Plex Mono', monospace;
                font-size: 0.78rem;
                margin: 0;
            }

            .setup-cards {
                display: flex;
                flex-direction: column;
                gap: 1rem;
            }

            .setup-card {
                background: #161b22;
                border: 1px solid #30363d;
                border-radius: 8px;
                padding: 1.25rem 1.5rem;
                transition: border-color 0.2s ease;
            }

            .setup-card.card-complete {
                border-color: #00d4aa44;
            }

            .card-title-row {
                display: flex;
                align-items: center;
                gap: 0.6rem;
                margin-bottom: 0.9rem;
            }

            .card-status-icon {
                width: 22px;
                height: 22px;
                border-radius: 50%;
                background: #21262d;
                border: 2px solid #30363d;
                display: flex;
                align-items: center;
                justify-content: center;
                font-size: 0.75rem;
                color: #8b949e;
                flex-shrink: 0;
                font-family: 'IBM Plex Mono', monospace;
            }

            .card-status-icon.icon-complete {
                background: #00d4aa;
                border-color: #00d4aa;
                color: #0a0e14;
                font-weight: 700;
            }

            .card-title {
                font-size: 0.95rem;
                font-weight: 600;
                margin: 0;
                flex: 1;
            }

            .optional-badge {
                font-family: 'IBM Plex Mono', monospace;
                font-size: 0.65rem;
                color: #8b949e;
                border: 1px solid #30363d;
                border-radius: 3px;
                padding: 0.1rem 0.4rem;
                letter-spacing: 0.05em;
            }

            .badge-done {
                font-family: 'IBM Plex Mono', monospace;
                font-size: 0.65rem;
                color: #00d4aa;
                border: 1px solid #00d4aa44;
                border-radius: 3px;
                padding: 0.1rem 0.4rem;
            }

            .card-verified {
                color: #00d4aa;
                font-size: 0.85rem;
                font-family: 'IBM Plex Mono', monospace;
                margin-bottom: 0.75rem;
            }

            .verified-detail {
                color: #8b949e;
            }

            .card-instructions p {
                color: #8b949e;
                font-size: 0.85rem;
                margin: 0 0 0.5rem;
            }

            .card-instructions {
                margin-bottom: 0.75rem;
            }

            .card-form {
                display: flex;
                gap: 0.5rem;
                margin-top: 0.5rem;
            }

            .card-input {
                flex: 1;
                background: #0d1117;
                border: 1px solid #30363d;
                border-radius: 6px;
                padding: 0.5rem 0.75rem;
                color: #e6edf3;
                font-family: 'IBM Plex Mono', monospace;
                font-size: 0.85rem;
                outline: none;
            }

            .card-input:focus {
                border-color: #00d4aa;
            }

            .card-button {
                background: #21262d;
                border: 1px solid #30363d;
                color: #e6edf3;
                padding: 0.5rem 1rem;
                border-radius: 6px;
                cursor: pointer;
                font-size: 0.85rem;
                white-space: nowrap;
                transition: all 0.15s ease;
            }

            .card-button:hover:not(:disabled) {
                border-color: #00d4aa;
                color: #00d4aa;
            }

            .card-button:disabled {
                opacity: 0.4;
                cursor: not-allowed;
            }

            .card-button-secondary {
                background: none;
                border: 1px solid #30363d;
                color: #8b949e;
                padding: 0.4rem 0.85rem;
                border-radius: 6px;
                cursor: pointer;
                font-size: 0.8rem;
                transition: all 0.15s ease;
            }

            .card-button-secondary:hover {
                border-color: #8b949e;
                color: #e6edf3;
            }

            .card-button-inline {
                background: none;
                border: none;
                color: #00d4aa;
                cursor: pointer;
                font-size: 0.8rem;
                padding: 0 0.25rem;
                margin-left: 0.5rem;
            }

            .card-error {
                color: #f97583;
                font-size: 0.82rem;
                margin-top: 0.5rem;
                font-family: 'IBM Plex Mono', monospace;
            }

            .card-warn {
                color: #e3b341;
                font-size: 0.82rem;
                margin-top: 0.5rem;
                font-family: 'IBM Plex Mono', monospace;
            }

            .card-loading {
                color: #8b949e;
                font-size: 0.82rem;
                font-family: 'IBM Plex Mono', monospace;
                margin-top: 0.5rem;
            }

            .card-inline-loading {
                color: #8b949e;
                font-size: 0.8rem;
                margin-left: 0.5rem;
            }

            .card-hint {
                color: #8b949e;
                font-size: 0.78rem;
            }

            .card-install-row {
                display: flex;
                align-items: center;
                gap: 0.75rem;
                margin-top: 0.5rem;
            }

            .card-manual {
                margin-top: 1rem;
                padding-top: 1rem;
                border-top: 1px solid #21262d;
            }

            .card-test-row {
                display: flex;
                align-items: center;
                gap: 0.75rem;
                margin-top: 1rem;
            }

            .test-pass {
                color: #00d4aa;
                font-size: 0.82rem;
                font-family: 'IBM Plex Mono', monospace;
            }

            .test-fail {
                color: #f97583;
                font-size: 0.82rem;
                font-family: 'IBM Plex Mono', monospace;
            }

            .code-block {
                background: #0d1117;
                border: 1px solid #21262d;
                border-radius: 4px;
                padding: 0.5rem 0.75rem;
                font-family: 'IBM Plex Mono', monospace;
                font-size: 0.8rem;
                color: #79c0ff;
                margin: 0.4rem 0;
                overflow-x: auto;
            }

            .auth-options {
                display: flex;
                gap: 1rem;
            }

            .auth-option {
                flex: 1;
            }

            .auth-option strong {
                font-size: 0.8rem;
                color: #8b949e;
            }

            .or-divider {
                color: #30363d;
                font-size: 0.75rem;
                display: flex;
                align-items: center;
                padding-top: 1.2rem;
            }

            .setup-ol {
                color: #8b949e;
                font-size: 0.85rem;
                margin: 0.5rem 0 0;
                padding-left: 1.25rem;
                line-height: 1.9;
            }

            .card-link {
                color: #00d4aa;
                font-size: 0.82rem;
                text-decoration: none;
            }

            .card-link:hover {
                text-decoration: underline;
            }

            .dashboard-row {
                margin-top: 2rem;
                text-align: center;
            }

            .dashboard-button {
                background: #21262d;
                border: 1px solid #30363d;
                color: #8b949e;
                padding: 0.75rem 2.5rem;
                border-radius: 8px;
                cursor: pointer;
                font-size: 1rem;
                font-weight: 600;
                transition: all 0.2s ease;
            }

            .dashboard-button.ready {
                background: linear-gradient(135deg, #00d4aa, #00a884);
                border-color: #00d4aa;
                color: #0a0e14;
            }

            .dashboard-button.ready:hover {
                opacity: 0.9;
                transform: translateY(-1px);
            }

            .dashboard-button-disabled {
                color: #484f58;
                font-size: 0.85rem;
                font-family: 'IBM Plex Mono', monospace;
            }
        """ ]
