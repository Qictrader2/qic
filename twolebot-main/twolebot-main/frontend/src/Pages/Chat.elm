module Pages.Chat exposing (Config, view)

import Dict
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as D
import Components.Markdown
import Types exposing (..)
import UI



-- ═══════════════════════════════════════════════════════════════════════════
-- CONFIG
-- ═══════════════════════════════════════════════════════════════════════════


type alias Config msg =
    { onNavigate : Route -> msg
    , onNewConversation : msg
    , onSelectConversation : String -> msg
    , onInputChange : String -> msg
    , onSendMessage : msg
    , onStartRename : String -> String -> msg
    , onRenameChange : String -> msg
    , onSubmitRename : msg
    , onCancelRename : msg
    , onStartVoice : msg
    , onStopVoice : msg
    , onCancelVoice : msg
    , onAttachFile : msg
    , onStartVideo : msg
    , onStopVideo : msg
    , onCancelVideo : msg
    , onConfirmDelete : String -> msg
    , onCancelDelete : msg
    , onDeleteConversation : String -> msg
    , onDismissError : String -> msg
    , onDismissNotification : Int -> msg
    , onRetryUpload : String -> String -> msg
    }



-- ═══════════════════════════════════════════════════════════════════════════
-- STYLE HELPERS
-- ═══════════════════════════════════════════════════════════════════════════


messageBubbleAttrs : Bool -> List (Attribute msg)
messageBubbleAttrs isUser =
    [ style "max-width" "80%"
    , style "padding" "0.6rem 0.8rem"
    , style "border-radius" "8px"
    , style "font-size" "0.8125rem"
    , style "line-height" "1.5"
    , style "white-space"
        (if isUser then
            "pre-wrap"

         else
            "normal"
        )
    , style "word-break" "break-word"
    , style "color" UI.colors.textPrimary
    , style "background-color"
        (if isUser then
            UI.colors.bgTertiary
         else
            UI.colors.accentDim
        )
    , style "border"
        (if isUser then
            "1px solid " ++ UI.colors.border
         else
            "1px solid rgba(0, 212, 170, 0.15)"
        )
    ]


messageRowAttrs : Bool -> List (Attribute msg)
messageRowAttrs isUser =
    [ style "display" "flex"
    , style "justify-content"
        (if isUser then
            "flex-end"
         else
            "flex-start"
        )
    ]


protocolBadge : Maybe String -> Html msg
protocolBadge protocol =
    case protocol of
        Just "web" ->
            span
                [ style "font-size" "0.625rem"
                , style "color" UI.colors.accent
                , style "background" UI.colors.accentDim
                , style "padding" "1px 4px"
                , style "border-radius" "2px"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "web" ]

        Just "telegram" ->
            span
                [ style "font-size" "0.625rem"
                , style "color" "#54a3ff"
                , style "background" "rgba(84, 163, 255, 0.1)"
                , style "padding" "1px 4px"
                , style "border-radius" "2px"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "tg" ]

        _ ->
            text ""


pulsingDot : String -> Html msg
pulsingDot color =
    div
        [ style "width" "10px"
        , style "height" "10px"
        , style "background-color" color
        , style "border-radius" "50%"
        , style "animation" "pulse 1s ease-in-out infinite"
        ]
        []


spinner : Html msg
spinner =
    div
        [ style "width" "16px"
        , style "height" "16px"
        , style "border" ("2px solid " ++ UI.colors.border)
        , style "border-top-color" UI.colors.accent
        , style "border-radius" "50%"
        , style "animation" "spin 0.8s linear infinite"
        ]
        []


thinkingIndicator : Html msg
thinkingIndicator =
    span
        [ style "display" "inline-flex"
        , style "align-items" "center"
        , style "gap" "0.4rem"
        , style "color" UI.colors.textMuted
        ]
        [ spinner
        , text "Thinking..."
        ]


transcribingIndicator : Html msg
transcribingIndicator =
    span
        [ style "display" "inline-flex"
        , style "align-items" "center"
        , style "gap" "0.4rem"
        , style "color" UI.colors.textMuted
        ]
        [ spinner
        , text "Transcribing..."
        ]


statusBar : String -> String -> List (Html msg) -> Html msg
statusBar bgColor borderColor children =
    div
        [ style "flex" "1"
        , style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.75rem"
        , style "padding" "0.5rem 0.75rem"
        , style "background-color" bgColor
        , style "border" ("1px solid " ++ borderColor)
        , style "border-radius" "6px"
        , style "min-height" "40px"
        ]
        children


inputBarButton : msg -> Bool -> String -> String -> String -> List (Attribute msg) -> List (Html msg) -> Html msg
inputBarButton onClickMsg isDisabled bg fg cursor extraAttrs children =
    button
        ([ onClick onClickMsg
         , disabled isDisabled
         , style "background" bg
         , style "color" fg
         , style "border" "none"
         , style "border-radius" "6px"
         , style "cursor" cursor
         , style "font-family" UI.fontBody
         , style "font-size" "0.8125rem"
         , style "font-weight" "600"
         , style "min-height" "40px"
         ]
            ++ extraAttrs
        )
        children


mediaButton : msg -> String -> String -> Html msg
mediaButton onClickMsg icon tooltip =
    button
        [ onClick onClickMsg
        , style "padding" "0.5rem"
        , style "background-color" "transparent"
        , style "color" UI.colors.textMuted
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "6px"
        , style "cursor" "pointer"
        , style "font-size" "1.1rem"
        , style "min-height" "40px"
        , style "min-width" "40px"
        , style "display" "flex"
        , style "align-items" "center"
        , style "justify-content" "center"
        , title tooltip
        ]
        [ text icon ]



-- ═══════════════════════════════════════════════════════════════════════════
-- VIEW
-- ═══════════════════════════════════════════════════════════════════════════


view : ChatPageState -> Config msg -> Html msg
view chatState config =
    div
        [ class
            (case chatState.activeChatId of
                Just _ ->
                    "chat-container chat-container--has-active"

                Nothing ->
                    "chat-container"
            )
        , style "display" "flex"
        , style "height" "calc(100vh - 140px)"
        , style "margin-top" "1rem"
        , style "gap" "0"
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "overflow" "hidden"
        ]
        [ viewSidebar chatState config
        , viewMainArea chatState config
        , viewNotifications chatState config
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- SIDEBAR
-- ═══════════════════════════════════════════════════════════════════════════


viewSidebar : ChatPageState -> Config msg -> Html msg
viewSidebar chatState config =
    div
        [ class
            (case chatState.activeChatId of
                Just _ ->
                    "chat-sidebar chat-sidebar--has-active"

                Nothing ->
                    "chat-sidebar"
            )
        , style "width" "280px"
        , style "min-width" "280px"
        , style "background-color" UI.colors.bgSecondary
        , style "border-right" ("1px solid " ++ UI.colors.border)
        , style "display" "flex"
        , style "flex-direction" "column"
        ]
        [ div
            [ style "padding" "0.75rem" ]
            [ button
                [ onClick config.onNewConversation
                , style "width" "100%"
                , style "padding" "0.5rem 1rem"
                , style "background" ("linear-gradient(135deg, " ++ UI.colors.accent ++ ", #00a884)")
                , style "color" UI.colors.bgPrimary
                , style "border" "none"
                , style "border-radius" "4px"
                , style "cursor" "pointer"
                , style "font-family" UI.fontBody
                , style "font-size" "0.8125rem"
                , style "font-weight" "600"
                , style "letter-spacing" "0.03em"
                ]
                [ text "+ New Chat" ]
            ]
        , div
            [ style "flex" "1"
            , style "overflow-y" "auto"
            , style "padding" "0 0.5rem 0.5rem"
            ]
            (case chatState.conversations of
                Success conversations ->
                    if List.isEmpty conversations then
                        [ div
                            [ style "padding" "2rem 1rem"
                            , style "color" UI.colors.textMuted
                            , style "text-align" "center"
                            , style "font-size" "0.8125rem"
                            ]
                            [ text "No conversations yet" ]
                        ]
                    else
                        List.map (viewConversationItem chatState config) conversations

                Loading ->
                    [ div
                        [ style "padding" "2rem 1rem"
                        , style "color" UI.colors.textMuted
                        , style "text-align" "center"
                        , style "font-size" "0.8125rem"
                        ]
                        [ text "Loading..." ]
                    ]

                Failure err ->
                    [ div
                        [ style "padding" "1rem"
                        , style "color" UI.colors.error
                        , style "font-size" "0.75rem"
                        ]
                        [ text err ]
                    ]

                NotAsked ->
                    []
            )
        ]


viewConversationItem : ChatPageState -> Config msg -> Conversation -> Html msg
viewConversationItem chatState config conv =
    let
        isActive =
            chatState.activeChatId == Just conv.id

        isRenaming =
            chatState.renamingConversationId == Just conv.id

        isConfirmingDelete =
            chatState.confirmingDeleteId == Just conv.id

        convState =
            getConversationState conv.id chatState

        hasWork =
            hasActiveWork convState
    in
    if isConfirmingDelete then
        viewDeleteConfirmation config conv

    else if isRenaming then
        viewRenameForm chatState config

    else
        div
            [ style "position" "relative"
            , style "margin-bottom" "2px"
            , class "conv-item"
            ]
            [ div
                [ onClick (config.onSelectConversation conv.id)
                , style "padding" "0.5rem 2rem 0.5rem 0.6rem"
                , style "cursor" "pointer"
                , style "border-radius" "4px"
                , style "transition" "background-color 0.1s"
                , style "background-color"
                    (if isActive then
                        UI.colors.bgTertiary
                     else
                        "transparent"
                    )
                , style "border-left"
                    (if isActive then
                        "2px solid " ++ UI.colors.accent
                     else
                        "2px solid transparent"
                    )
                , onDoubleClick (config.onStartRename conv.id conv.name)
                ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.4rem"
                    , style "margin-bottom" "0.15rem"
                    ]
                    [ protocolBadge conv.protocol
                    , if hasWork then
                        span
                            [ style "width" "6px"
                            , style "height" "6px"
                            , style "background-color" UI.colors.accent
                            , style "border-radius" "50%"
                            , style "animation" "pulse 2s ease-in-out infinite"
                            , style "flex-shrink" "0"
                            ]
                            []
                      else
                        text ""
                    , span
                        [ style "font-size" "0.8125rem"
                        , style "color"
                            (if isActive then
                                UI.colors.textPrimary
                             else
                                UI.colors.textSecondary
                            )
                        , style "font-weight"
                            (if isActive then
                                "500"
                             else
                                "400"
                            )
                        , style "overflow" "hidden"
                        , style "text-overflow" "ellipsis"
                        , style "white-space" "nowrap"
                        , style "flex" "1"
                        ]
                        [ text conv.name ]
                    ]
                , case conv.lastMessagePreview of
                    Just preview ->
                        div
                            [ style "font-size" "0.6875rem"
                            , style "color" UI.colors.textMuted
                            , style "overflow" "hidden"
                            , style "text-overflow" "ellipsis"
                            , style "white-space" "nowrap"
                            ]
                            [ text preview ]

                    Nothing ->
                        text ""
                ]
            , button
                [ onClick (config.onConfirmDelete conv.id)
                , class "conv-delete-btn"
                , style "position" "absolute"
                , style "top" "0.4rem"
                , style "right" "0.3rem"
                , style "width" "22px"
                , style "height" "22px"
                , style "padding" "0"
                , style "background" "transparent"
                , style "color" UI.colors.textMuted
                , style "border" "none"
                , style "border-radius" "3px"
                , style "cursor" "pointer"
                , style "font-size" "0.75rem"
                , style "display" "flex"
                , style "align-items" "center"
                , style "justify-content" "center"
                , style "opacity" "0"
                , style "transition" "opacity 0.15s"
                , title "Delete conversation"
                ]
                [ text "\u{00D7}" ]
            ]


viewDeleteConfirmation : Config msg -> Conversation -> Html msg
viewDeleteConfirmation config conv =
    div
        [ style "padding" "0.5rem"
        , style "background-color" UI.colors.errorDim
        , style "border" ("1px solid " ++ UI.colors.error)
        , style "border-radius" "4px"
        , style "margin-bottom" "2px"
        ]
        [ div
            [ style "font-size" "0.75rem"
            , style "color" UI.colors.error
            , style "margin-bottom" "0.4rem"
            , style "font-weight" "500"
            ]
            [ text "Delete this conversation?" ]
        , div
            [ style "font-size" "0.6875rem"
            , style "color" UI.colors.textMuted
            , style "margin-bottom" "0.5rem"
            , style "overflow" "hidden"
            , style "text-overflow" "ellipsis"
            , style "white-space" "nowrap"
            ]
            [ text conv.name ]
        , div
            [ style "display" "flex", style "gap" "0.25rem" ]
            [ button
                [ onClick (config.onDeleteConversation conv.id)
                , style "flex" "1"
                , style "padding" "0.3rem"
                , style "background-color" UI.colors.error
                , style "color" "#fff"
                , style "border" "none"
                , style "border-radius" "3px"
                , style "cursor" "pointer"
                , style "font-size" "0.6875rem"
                , style "font-weight" "600"
                , style "font-family" UI.fontBody
                ]
                [ text "Delete" ]
            , button
                [ onClick config.onCancelDelete
                , style "flex" "1"
                , style "padding" "0.3rem"
                , style "background-color" "transparent"
                , style "color" UI.colors.textMuted
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "3px"
                , style "cursor" "pointer"
                , style "font-size" "0.6875rem"
                , style "font-family" UI.fontBody
                ]
                [ text "Cancel" ]
            ]
        ]


viewRenameForm : ChatPageState -> Config msg -> Html msg
viewRenameForm chatState config =
    div
        [ style "padding" "0.5rem"
        , style "background-color" UI.colors.bgTertiary
        , style "border-radius" "4px"
        , style "margin-bottom" "2px"
        ]
        [ input
            [ value chatState.renameText
            , onInput config.onRenameChange
            , Html.Events.preventDefaultOn "keydown"
                (D.field "key" D.string
                    |> D.andThen
                        (\key ->
                            case key of
                                "Enter" ->
                                    D.succeed ( config.onSubmitRename, True )

                                "Escape" ->
                                    D.succeed ( config.onCancelRename, True )

                                _ ->
                                    D.fail "ignore"
                        )
                )
            , id "rename-input"
            , style "width" "100%"
            , style "background-color" UI.colors.bgSurface
            , style "color" UI.colors.textPrimary
            , style "border" ("1px solid " ++ UI.colors.accent)
            , style "border-radius" "3px"
            , style "padding" "0.3rem 0.5rem"
            , style "font-family" UI.fontBody
            , style "font-size" "0.8125rem"
            , style "outline" "none"
            , style "box-sizing" "border-box"
            ]
            []
        , div
            [ style "display" "flex"
            , style "gap" "0.25rem"
            , style "margin-top" "0.25rem"
            ]
            [ button
                [ onClick config.onSubmitRename
                , style "flex" "1"
                , style "padding" "0.2rem"
                , style "background-color" UI.colors.accent
                , style "color" UI.colors.bgPrimary
                , style "border" "none"
                , style "border-radius" "2px"
                , style "cursor" "pointer"
                , style "font-size" "0.6875rem"
                ]
                [ text "Save" ]
            , button
                [ onClick config.onCancelRename
                , style "flex" "1"
                , style "padding" "0.2rem"
                , style "background-color" "transparent"
                , style "color" UI.colors.textMuted
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "2px"
                , style "cursor" "pointer"
                , style "font-size" "0.6875rem"
                ]
                [ text "Cancel" ]
            ]
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- MAIN CHAT AREA
-- ═══════════════════════════════════════════════════════════════════════════


viewMainArea : ChatPageState -> Config msg -> Html msg
viewMainArea chatState config =
    div
        [ class
            (case chatState.activeChatId of
                Just _ ->
                    "chat-main chat-main--active"

                Nothing ->
                    "chat-main"
            )
        , style "flex" "1"
        , style "display" "flex"
        , style "flex-direction" "column"
        , style "background-color" UI.colors.bgPrimary
        , style "min-width" "0"
        ]
        (case chatState.activeChatId of
            Nothing ->
                [ viewEmptyState ]

            Just cid ->
                let
                    conv =
                        getConversationState cid chatState
                in
                [ viewChatHeader config
                , viewMessages conv
                , viewUploadQueue cid conv config
                , viewErrorBanner cid conv config
                , viewInputBar conv config
                ]
        )


viewChatHeader : Config msg -> Html msg
viewChatHeader config =
    div
        [ class "chat-back-header"
        , style "display" "none"
        , style "align-items" "center"
        , style "padding" "0.5rem 0.75rem"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        , style "background-color" UI.colors.bgSecondary
        ]
        [ button
            [ onClick (config.onNavigate (ChatRoute Nothing))
            , style "background" "none"
            , style "border" "none"
            , style "color" UI.colors.accent
            , style "cursor" "pointer"
            , style "font-family" UI.fontBody
            , style "font-size" "0.875rem"
            , style "padding" "0.25rem 0.5rem"
            , style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.25rem"
            ]
            [ span [] [ text "\u{2190}" ]
            , text "Chats"
            ]
        ]


viewEmptyState : Html msg
viewEmptyState =
    div
        [ style "flex" "1"
        , style "display" "flex"
        , style "flex-direction" "column"
        , style "align-items" "center"
        , style "justify-content" "center"
        , style "gap" "1rem"
        ]
        [ div
            [ style "width" "48px"
            , style "height" "48px"
            , style "background" ("linear-gradient(135deg, " ++ UI.colors.accent ++ " 0%, #00a884 100%)")
            , style "clip-path" "polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)"
            , style "opacity" "0.4"
            ]
            []
        , div
            [ style "color" UI.colors.textMuted
            , style "font-size" "0.875rem"
            ]
            [ text "Select a conversation or start a new one" ]
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- MESSAGES
-- ═══════════════════════════════════════════════════════════════════════════


viewMessages : ChatConversationState -> Html msg
viewMessages conv =
    let
        allMessages =
            conv.messages ++ conv.pendingOutbound
    in
    div
        [ id "chat-messages"
        , style "flex" "1"
        , style "overflow-y" "auto"
        , style "padding" "1rem"
        , style "display" "flex"
        , style "flex-direction" "column"
        , style "gap" "0.75rem"
        ]
        (viewMessagesGrouped allMessages
            ++ viewStreamingIndicator conv.activity
        )


isToolCallOnly : ChatMessage -> Maybe String
isToolCallOnly msg =
    if msg.direction == Outbound then
        let
            trimmed =
                String.trim msg.content
        in
        if String.startsWith "\u{1F527} Using tool: " trimmed then
            Just (String.replace "\u{1F527} Using tool: " "" trimmed |> String.trim)

        else
            Nothing

    else
        Nothing


viewMessagesGrouped : List ChatMessage -> List (Html msg)
viewMessagesGrouped messages =
    viewMessagesGroupedHelp messages [] []
        |> List.reverse


viewMessagesGroupedHelp : List ChatMessage -> List String -> List (Html msg) -> List (Html msg)
viewMessagesGroupedHelp messages toolGroup result =
    case messages of
        [] ->
            flushToolMessages toolGroup result

        msg :: rest ->
            case isToolCallOnly msg of
                Just toolName ->
                    viewMessagesGroupedHelp rest (toolGroup ++ [ toolName ]) result

                Nothing ->
                    let
                        flushed =
                            flushToolMessages toolGroup result
                    in
                    viewMessagesGroupedHelp rest [] (viewMessage msg :: flushed)


flushToolMessages : List String -> List (Html msg) -> List (Html msg)
flushToolMessages group result =
    case group of
        [] ->
            result

        [ single ] ->
            div (messageRowAttrs False)
                [ div (messageBubbleAttrs False)
                    [ span
                        [ style "font-size" "0.7rem"
                        , style "color" UI.colors.textMuted
                        ]
                        [ text ("\u{1F527} " ++ single) ]
                    ]
                ]
                :: result

        _ ->
            div (messageRowAttrs False)
                [ div (messageBubbleAttrs False)
                    [ viewToolCallGroup group ]
                ]
                :: result


viewStreamingIndicator : ChatActivity -> List (Html msg)
viewStreamingIndicator activity =
    case activity of
        ChatTranscribing ->
            [ div (messageRowAttrs False)
                [ div (messageBubbleAttrs False)
                    [ transcribingIndicator ]
                ]
            ]

        ChatAwaitingResponse _ ->
            [ div (messageRowAttrs False)
                [ div (messageBubbleAttrs False)
                    [ thinkingIndicator ]
                ]
            ]

        ChatStreaming { buffer } ->
            [ div (messageRowAttrs False)
                [ div (messageBubbleAttrs False)
                    (if String.isEmpty buffer then
                        [ thinkingIndicator ]
                     else
                        renderMessageContent True buffer ++ [ text " |" ]
                    )
                ]
            ]

        _ ->
            []


viewMessage : ChatMessage -> Html msg
viewMessage msg =
    let
        isUser =
            msg.direction == Inbound
    in
    div (messageRowAttrs isUser)
        [ div (messageBubbleAttrs isUser)
            (renderMessageContent (not isUser) msg.content ++ viewMediaAttachments msg.attachments)
        ]


viewMediaAttachments : List MediaAttachment -> List (Html msg)
viewMediaAttachments attachments =
    List.concatMap viewMediaAttachment attachments


viewMediaAttachment : MediaAttachment -> List (Html msg)
viewMediaAttachment attachment =
    case attachment of
        AudioAttachment { path } ->
            [ div [ style "margin-top" "0.5rem" ]
                [ audio
                    [ src ("/api/media/" ++ path)
                    , controls True
                    , style "max-width" "100%"
                    ]
                    []
                ]
            ]

        VideoAttachment { path } ->
            [ div [ style "margin-top" "0.5rem" ]
                [ video
                    [ src ("/api/media/" ++ path)
                    , controls True
                    , style "max-width" "100%"
                    , style "border-radius" "4px"
                    ]
                    []
                ]
            ]

        ImageAttachment { path } ->
            [ div [ style "margin-top" "0.5rem" ]
                [ img
                    [ src ("/api/media/" ++ path)
                    , style "max-width" "100%"
                    , style "border-radius" "4px"
                    ]
                    []
                ]
            ]

        FileAttachment { path, name } ->
            [ div [ style "margin-top" "0.5rem" ]
                [ a
                    [ Html.Attributes.href ("/api/media/" ++ path)
                    , Html.Attributes.target "_blank"
                    , style "color" UI.colors.accent
                    , style "text-decoration" "none"
                    , style "display" "inline-flex"
                    , style "align-items" "center"
                    , style "gap" "0.4rem"
                    , style "padding" "0.3rem 0.5rem"
                    , style "background" UI.colors.accentDim
                    , style "border-radius" "4px"
                    , style "font-size" "0.75rem"
                    ]
                    [ text ("\u{1F4C4} " ++ name) ]
                ]
            ]



-- ═══════════════════════════════════════════════════════════════════════════
-- UPLOAD QUEUE
-- ═══════════════════════════════════════════════════════════════════════════


viewUploadQueue : String -> ChatConversationState -> Config msg -> Html msg
viewUploadQueue convId conv config =
    let
        tasks =
            conv.uploads
    in
    if List.isEmpty tasks then
        text ""

    else
        div
            [ style "padding" "0.25rem 0.75rem"
            , style "border-top" ("1px solid " ++ UI.colors.border)
            , style "background-color" UI.colors.bgSecondary
            , style "display" "flex"
            , style "flex-wrap" "wrap"
            , style "gap" "0.4rem"
            ]
            (List.map (viewUploadTask convId config) tasks)


viewUploadTask : String -> Config msg -> UploadTask -> Html msg
viewUploadTask convId config task =
    let
        label =
            case task.media of
                UploadVoice _ -> "Voice"
                UploadVideo _ -> "Video"
                UploadFile f -> f.name
    in
    case task.status of
        Uploading ->
            div
                [ style "display" "inline-flex"
                , style "align-items" "center"
                , style "gap" "0.4rem"
                , style "padding" "0.2rem 0.5rem"
                , style "background" UI.colors.bgTertiary
                , style "border-radius" "4px"
                , style "font-size" "0.6875rem"
                , style "color" UI.colors.textMuted
                ]
                [ spinner, text label ]

        UploadSucceeded _ ->
            text ""

        UploadFailed errMsg ->
            div
                [ style "display" "inline-flex"
                , style "align-items" "center"
                , style "gap" "0.4rem"
                , style "padding" "0.2rem 0.5rem"
                , style "background" UI.colors.errorDim
                , style "border-radius" "4px"
                , style "font-size" "0.6875rem"
                , style "color" UI.colors.error
                ]
                [ text (label ++ " failed")
                , button
                    [ onClick (config.onRetryUpload convId task.id)
                    , style "background" "none"
                    , style "border" "none"
                    , style "color" UI.colors.accent
                    , style "cursor" "pointer"
                    , style "font-size" "0.625rem"
                    , style "text-decoration" "underline"
                    , style "padding" "0"
                    , style "font-family" UI.fontBody
                    , title errMsg
                    ]
                    [ text "retry" ]
                ]



-- ═══════════════════════════════════════════════════════════════════════════
-- ERROR BANNER
-- ═══════════════════════════════════════════════════════════════════════════


viewErrorBanner : String -> ChatConversationState -> Config msg -> Html msg
viewErrorBanner convId conv config =
    case conv.activity of
        ChatError errInfo ->
            div
                [ style "padding" "0.5rem 0.75rem"
                , style "background-color" UI.colors.errorDim
                , style "border-top" ("1px solid " ++ UI.colors.error)
                , style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "font-size" "0.8125rem"
                ]
                [ span [ style "color" UI.colors.error, style "flex" "1" ]
                    [ text (chatErrorText errInfo.error) ]
                , button
                    [ onClick (config.onDismissError convId)
                    , style "background" "none"
                    , style "border" "none"
                    , style "color" UI.colors.textMuted
                    , style "cursor" "pointer"
                    , style "font-size" "0.875rem"
                    , style "padding" "0.2rem"
                    , style "font-family" UI.fontBody
                    ]
                    [ text "\u{00D7}" ]
                ]

        _ ->
            text ""


chatErrorText : ChatError -> String
chatErrorText err =
    case err of
        SendFailed msg ->
            "Failed to send: " ++ msg

        StreamInterrupted msg ->
            "Stream interrupted: " ++ msg

        ConnectionLost ->
            "Connection lost. Reconnecting..."

        MediaUploadFailed msg ->
            msg



-- ═══════════════════════════════════════════════════════════════════════════
-- NOTIFICATIONS
-- ═══════════════════════════════════════════════════════════════════════════


viewNotifications : ChatPageState -> Config msg -> Html msg
viewNotifications chatState config =
    if List.isEmpty chatState.notifications then
        text ""

    else
        div
            [ style "position" "fixed"
            , style "bottom" "1rem"
            , style "right" "1rem"
            , style "display" "flex"
            , style "flex-direction" "column"
            , style "gap" "0.5rem"
            , style "z-index" "100"
            , style "max-width" "320px"
            ]
            (List.map (viewNotification chatState config) (List.take 5 chatState.notifications))


viewNotification : ChatPageState -> Config msg -> ChatNotification -> Html msg
viewNotification chatState config note =
    let
        convName =
            case chatState.conversations of
                Success convs ->
                    List.filter (\c -> c.id == note.conversationId) convs
                        |> List.head
                        |> Maybe.map .name
                        |> Maybe.withDefault "Conversation"

                _ ->
                    "Conversation"

        ( icon, message ) =
            case note.kind of
                ResponseComplete { preview } ->
                    ( "\u{2705}", convName ++ ": " ++ String.left 60 preview )

                MediaUploadComplete ->
                    ( "\u{1F4E4}", convName ++ ": Upload complete" )

                ChatErrorNotification err ->
                    ( "\u{26A0}", convName ++ ": " ++ chatErrorText err )
    in
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "6px"
        , style "padding" "0.6rem 0.75rem"
        , style "font-size" "0.8125rem"
        , style "color" UI.colors.textPrimary
        , style "display" "flex"
        , style "align-items" "flex-start"
        , style "gap" "0.5rem"
        , style "box-shadow" "0 2px 8px rgba(0,0,0,0.3)"
        , style "cursor" "pointer"
        , onClick (config.onSelectConversation note.conversationId)
        ]
        [ span [] [ text icon ]
        , span [ style "flex" "1", style "line-height" "1.4" ] [ text message ]
        , button
            [ stopPropagationOn "click" (D.succeed ( config.onDismissNotification note.id, True ))
            , style "background" "none"
            , style "border" "none"
            , style "color" UI.colors.textMuted
            , style "cursor" "pointer"
            , style "font-size" "0.875rem"
            , style "padding" "0"
            , style "font-family" UI.fontBody
            ]
            [ text "\u{00D7}" ]
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- INPUT BAR
-- ═══════════════════════════════════════════════════════════════════════════


viewInputBar : ChatConversationState -> Config msg -> Html msg
viewInputBar conv config =
    div
        [ style "padding" "0.75rem"
        , style "border-top" ("1px solid " ++ UI.colors.border)
        , style "background-color" UI.colors.bgSecondary
        , style "display" "flex"
        , style "gap" "0.5rem"
        , style "align-items" "flex-end"
        ]
        (case conv.activity of
            ChatComposing ComposingVoice ->
                viewRecordingBar config

            ChatComposing ComposingVideo ->
                viewVideoRecordingBar config

            ChatObserving ->
                viewObserverBar

            _ ->
                viewTextInputBar conv config
        )


viewRecordingBar : Config msg -> List (Html msg)
viewRecordingBar config =
    [ statusBar UI.colors.errorDim UI.colors.error
        [ pulsingDot UI.colors.error
        , span
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.error
            , style "font-weight" "500"
            ]
            [ text "Recording..." ]
        ]
    , inputBarButton config.onCancelVoice
        False
        UI.colors.bgTertiary
        UI.colors.textMuted
        "pointer"
        [ style "padding" "0.5rem 0.75rem" ]
        [ text "Cancel" ]
    , inputBarButton config.onStopVoice
        False
        ("linear-gradient(135deg, " ++ UI.colors.error ++ ", #dc2626)")
        "#fff"
        "pointer"
        [ style "padding" "0.5rem 1rem" ]
        [ text "Stop & Send" ]
    ]
    
    
viewVideoRecordingBar : Config msg -> List (Html msg)
viewVideoRecordingBar config =
    [ statusBar UI.colors.errorDim UI.colors.error
        [ pulsingDot UI.colors.error
        , span
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.error
            , style "font-weight" "500"
            ]
            [ text "Recording video..." ]
        ]
    , inputBarButton config.onCancelVideo
        False
        UI.colors.bgTertiary
        UI.colors.textMuted
        "pointer"
        [ style "padding" "0.5rem 0.75rem" ]
        [ text "Cancel" ]
    , inputBarButton config.onStopVideo
        False
        ("linear-gradient(135deg, " ++ UI.colors.error ++ ", #dc2626)")
        "#fff"
        "pointer"
        [ style "padding" "0.5rem 1rem" ]
        [ text "Stop & Send" ]
    ]


viewObserverBar : List (Html msg)
viewObserverBar =
    [ statusBar "rgba(84, 163, 255, 0.08)" "rgba(84, 163, 255, 0.3)"
        [ span
            [ style "font-size" "0.8125rem"
            , style "color" "#54a3ff"
            ]
            [ text "Telegram conversation \u{2014} view only" ]
        ]
    ]


viewTextInputBar : ChatConversationState -> Config msg -> List (Html msg)
viewTextInputBar conv config =
    let
        hasText =
            not (String.isEmpty (String.trim conv.inputText))

        canSend =
            hasText && canSendMessage conv.activity

        isBotWorking =
            isStreamingOrWaiting conv.activity
    in
    [ label
        [ Html.Attributes.for "chat-file-input"
        , style "padding" "0.5rem"
        , style "background-color" "transparent"
        , style "color" UI.colors.textMuted
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "6px"
        , style "cursor" "pointer"
        , style "font-size" "1.1rem"
        , style "min-height" "40px"
        , style "min-width" "40px"
        , style "display" "flex"
        , style "align-items" "center"
        , style "justify-content" "center"
        , title "Attach file"
        ]
        [ text "\u{1F4CE}" ]
    , div
        [ style "flex" "1"
        , style "display" "flex"
        , style "flex-direction" "column"
        , style "gap" "0.25rem"
        ]
        [ textarea
            [ value conv.inputText
            , onInput config.onInputChange
            , placeholder
                (if isBotWorking then
                    "Bot is responding\u{2026} type your next message"
                 else
                    "Type a message..."
                )
            , style "flex" "1"
            , style "resize" "none"
            , style "min-height" "40px"
            , style "max-height" "120px"
            , style "padding" "0.5rem 0.75rem"
            , style "background-color" UI.colors.bgSurface
            , style "color" UI.colors.textPrimary
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "6px"
            , style "font-family" UI.fontBody
            , style "font-size" "0.8125rem"
            , style "line-height" "1.4"
            , style "outline" "none"
            , rows 1
            , id "chat-input"
            ]
            []
        ]
    , if hasText then
        inputBarButton config.onSendMessage
            (not canSend)
            (if canSend then
                "linear-gradient(135deg, " ++ UI.colors.accent ++ ", #00a884)"
             else
                UI.colors.bgTertiary
            )
            (if canSend then
                UI.colors.bgPrimary
             else
                UI.colors.textMuted
            )
            (if canSend then
                "pointer"
             else
                "not-allowed"
            )
            [ style "padding" "0.5rem 1rem"
            , id "chat-send-btn"
            ]
            [ span [ style "display" "flex", style "align-items" "center", style "gap" "0.4rem" ]
                [ text "Send"
                , span
                    [ class "desktop-shortcut-hint"
                    , style "font-size" "0.625rem"
                    , style "opacity" "0.6"
                    , style "font-weight" "400"
                    , title "Enter to send, Shift+Enter for newline"
                    ]
                    [ text "\u{21B5}" ]
                ]
            ]

      else
        div [ style "display" "flex", style "gap" "0.5rem" ]
            [ mediaButton config.onStartVideo "\u{1F3A5}" "Record video"
            , mediaButton config.onStartVoice "\u{1F3A4}" "Voice message"
            ]
    ]



-- ═══════════════════════════════════════════════════════════════════════════
-- MESSAGE CONTENT RENDERING (tool calls, thinking blocks)
-- ═══════════════════════════════════════════════════════════════════════════


type ContentSegment
    = PlainText String
    | ToolCall String
    | ThinkingBlock String


renderMessageContent : Bool -> String -> List (Html msg)
renderMessageContent asMarkdown content =
    if not (String.contains "\u{1F527}" content || String.contains "\u{1F4AD}" content) then
        if asMarkdown then
            [ Components.Markdown.view content ]

        else
            [ text content ]

    else
        content
            |> splitIntoSegments
            |> renderSegments asMarkdown


splitIntoSegments : String -> List ContentSegment
splitIntoSegments content =
    let
        lines =
            String.split "\n" content
    in
    groupLines lines [] []
        |> List.reverse


groupLines : List String -> List String -> List ContentSegment -> List ContentSegment
groupLines lines currentText segments =
    case lines of
        [] ->
            let
                textStr =
                    String.join "\n" (List.reverse currentText)
            in
            if String.isEmpty textStr then
                segments

            else
                PlainText textStr :: segments

        line :: rest ->
            let
                trimmed =
                    String.trim line
            in
            if String.startsWith "\u{1F527} Using tool: " trimmed then
                let
                    textStr =
                        String.join "\n" (List.reverse currentText)
                            |> String.trimRight

                    toolName =
                        String.replace "\u{1F527} Using tool: " "" trimmed
                            |> String.trim

                    withText =
                        if String.isEmpty textStr then
                            segments

                        else
                            PlainText textStr :: segments
                in
                groupLines rest [] (ToolCall toolName :: withText)

            else if String.startsWith "\u{1F4AD} Thinking: " trimmed then
                let
                    textStr =
                        String.join "\n" (List.reverse currentText)
                            |> String.trimRight

                    thinkText =
                        String.replace "\u{1F4AD} Thinking: " "" trimmed

                    withText =
                        if String.isEmpty textStr then
                            segments

                        else
                            PlainText textStr :: segments
                in
                groupLines rest [] (ThinkingBlock thinkText :: withText)

            else if String.startsWith "\u{1F4CA} Tool result from " trimmed then
                groupLines rest currentText segments

            else
                groupLines rest (line :: currentText) segments


renderSegments : Bool -> List ContentSegment -> List (Html msg)
renderSegments asMarkdown segments =
    renderSegmentsHelp asMarkdown segments [] []
        |> List.reverse


renderSegmentsHelp : Bool -> List ContentSegment -> List String -> List (Html msg) -> List (Html msg)
renderSegmentsHelp asMarkdown segments toolGroup result =
    case segments of
        [] ->
            flushToolGroup toolGroup result

        (ToolCall name) :: rest ->
            renderSegmentsHelp asMarkdown rest (toolGroup ++ [ name ]) result

        segment :: rest ->
            let
                flushed =
                    flushToolGroup toolGroup result

                rendered =
                    renderSegment asMarkdown segment
            in
            renderSegmentsHelp asMarkdown rest [] (rendered :: flushed)


flushToolGroup : List String -> List (Html msg) -> List (Html msg)
flushToolGroup group result =
    case group of
        [] ->
            result

        _ ->
            viewToolCallGroup group :: result


renderSegment : Bool -> ContentSegment -> Html msg
renderSegment asMarkdown segment =
    case segment of
        PlainText str ->
            if asMarkdown then
                Components.Markdown.view str

            else
                text str

        ToolCall toolName ->
            viewToolCallGroup [ toolName ]

        ThinkingBlock thinkText ->
            Html.node "details"
                [ style "margin" "2px 0"
                , style "font-size" "0.7rem"
                ]
                [ Html.node "summary"
                    [ style "cursor" "pointer"
                    , style "color" UI.colors.textMuted
                    , style "user-select" "none"
                    ]
                    [ text "\u{1F4AD} thinking..." ]
                , div
                    [ style "padding" "0.25rem 0 0.25rem 1rem"
                    , style "color" UI.colors.textMuted
                    , style "white-space" "pre-wrap"
                    ]
                    [ text thinkText ]
                ]


viewToolCallGroup : List String -> Html msg
viewToolCallGroup toolNames =
    let
        count =
            List.length toolNames

        summaryText =
            "\u{1F527} " ++ String.fromInt count ++ " tool call" ++ (if count /= 1 then "s" else "")
    in
    Html.node "details"
        [ style "margin" "4px 0"
        , style "font-size" "0.7rem"
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "6px"
        , style "padding" "2px 8px"
        ]
        [ Html.node "summary"
            [ style "cursor" "pointer"
            , style "color" UI.colors.textMuted
            , style "user-select" "none"
            ]
            [ text summaryText ]
        , div [ style "padding" "2px 0 2px 12px" ]
            (List.map
                (\name ->
                    div
                        [ style "color" UI.colors.textMuted
                        , style "padding" "1px 0"
                        ]
                        [ text ("\u{2022} " ++ name) ]
                )
                toolNames
            )
        ]
