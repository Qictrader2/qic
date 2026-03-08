module Pages.Messages exposing (view)

import Components.Markdown
import Components.Pagination
import Dict exposing (Dict)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


view : Maybe String -> Maybe String -> RemoteData (List ChatSummary) -> RemoteData MessagesPage -> String -> msg -> (Int -> msg) -> (String -> msg) -> msg -> msg -> Html msg
view selectedChat topicFilter chats messagesPage searchQuery onBack onPageChange onSearchChange onSearchSubmit onRefresh =
    let
        headerText =
            case selectedChat of
                Just chatId ->
                    case topicFilter of
                        Just "none" ->
                            "Chat " ++ chatId ++ " · General"

                        Just tid ->
                            "Chat " ++ chatId ++ " · Topic " ++ tid

                        Nothing ->
                            "Chat " ++ chatId

                Nothing ->
                    "Conversations"
    in
    div []
        [ UI.pageHeader headerText
            [ case selectedChat of
                Just _ ->
                    UI.row "0.75rem"
                        [ UI.backButton onBack
                        , UI.button_ [ onClick onRefresh ] "Refresh"
                        ]
                Nothing ->
                    text ""
            ]
        , case selectedChat of
            Nothing ->
                viewChatList chats

            Just chatId ->
                viewMessageList chatId messagesPage searchQuery onPageChange onSearchChange onSearchSubmit
        ]


viewChatList : RemoteData (List ChatSummary) -> Html msg
viewChatList chats =
    case chats of
        Loading ->
            UI.loadingSpinner

        Failure err ->
            UI.card [] [ text ("Error: " ++ err) ]

        NotAsked ->
            UI.loadingText "Loading chats..."

        Success chatList ->
            if List.isEmpty chatList then
                UI.card []
                    [ UI.emptyStateWithIcon "—" "No conversations yet" ]
            else
                let
                    grouped =
                        groupChatsByChat chatList
                in
                UI.col "1.5rem" (List.map viewChatGroup grouped)


{-| Group ChatSummary items by chat_id, preserving order of first appearance.
Returns list of (chatId, displayName, items) tuples.
-}
groupChatsByChat : List ChatSummary -> List ( String, String, List ChatSummary )
groupChatsByChat chats =
    let
        addToGroup : ChatSummary -> ( List String, Dict String ( String, List ChatSummary ) ) -> ( List String, Dict String ( String, List ChatSummary ) )
        addToGroup chat ( order, dict ) =
            let
                label =
                    chat.displayName
                        |> orElse chat.username
                        |> Maybe.withDefault ("Chat " ++ chat.chatId)
            in
            case Dict.get chat.chatId dict of
                Just ( existingLabel, items ) ->
                    ( order, Dict.insert chat.chatId ( existingLabel, items ++ [ chat ] ) dict )

                Nothing ->
                    ( order ++ [ chat.chatId ], Dict.insert chat.chatId ( label, [ chat ] ) dict )

        ( orderedKeys, grouped ) =
            List.foldl addToGroup ( [], Dict.empty ) chats
    in
    orderedKeys
        |> List.filterMap
            (\key ->
                Dict.get key grouped
                    |> Maybe.map (\( label, items ) -> ( key, label, items ))
            )


viewChatGroup : ( String, String, List ChatSummary ) -> Html msg
viewChatGroup ( chatId, displayName, items ) =
    let
        totalMessages =
            List.map .messageCount items |> List.sum

        hasTopics =
            List.length items > 1 || List.any (\c -> c.topicId /= Nothing) items
    in
    div
        [ style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "overflow" "hidden"
        ]
        [ -- Chat header
          div
            [ style "padding" "1rem 1.25rem"
            , style "border-bottom" ("1px solid " ++ UI.colors.border)
            ]
            [ div
                [ style "display" "flex"
                , style "justify-content" "space-between"
                , style "align-items" "center"
                ]
                [ div []
                    [ div
                        [ style "font-weight" "600"
                        , style "font-size" "0.875rem"
                        , style "color" UI.colors.textPrimary
                        , style "letter-spacing" "0.02em"
                        ]
                        [ text displayName ]
                    , div
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.6875rem"
                        , style "color" UI.colors.textMuted
                        , style "margin-top" "0.25rem"
                        , style "letter-spacing" "0.05em"
                        ]
                        [ text (chatId ++ " · " ++ String.fromInt totalMessages ++ " messages") ]
                    ]
                , if not hasTopics then
                    div
                        [ style "width" "24px"
                        , style "height" "24px"
                        , style "border" ("1px solid " ++ UI.colors.border)
                        , style "border-radius" "2px"
                        , style "display" "flex"
                        , style "align-items" "center"
                        , style "justify-content" "center"
                        , style "color" UI.colors.textMuted
                        , style "font-size" "0.75rem"
                        , style "cursor" "pointer"
                        ]
                        [ text "→" ]
                  else
                    text ""
                ]
            ]
        , -- Topic list (or single click target)
          if hasTopics then
            div [] (List.indexedMap (viewTopicItem chatId) items)
          else
            -- Single chat without topics - whole card is clickable
            text ""
        ]
        |> wrapClickable hasTopics chatId


wrapClickable : Bool -> String -> Html msg -> Html msg
wrapClickable hasTopics chatId inner =
    if hasTopics then
        inner
    else
        a
            [ href (topicHref chatId Nothing)
            , style "cursor" "pointer"
            , style "transition" "all 0.15s ease"
            , style "text-decoration" "none"
            , style "color" "inherit"
            , style "display" "block"
            ]
            [ inner ]


topicHref : String -> Maybe Int -> String
topicHref chatId maybeTopicId =
    case maybeTopicId of
        Just tid ->
            "/messages/" ++ chatId ++ "?topic=" ++ String.fromInt tid

        Nothing ->
            "/messages/" ++ chatId ++ "?topic=none"


viewTopicItem : String -> Int -> ChatSummary -> Html msg
viewTopicItem chatId index chat =
    let
        topicLabel =
            case chat.topicId of
                Just tid ->
                    "Topic " ++ String.fromInt tid

                Nothing ->
                    "General"

        topicIcon =
            case chat.topicId of
                Just _ ->
                    "#"

                Nothing ->
                    "·"
    in
    a
        ([ href (topicHref chatId chat.topicId)
         , style "padding" "0.75rem 1.25rem"
         , style "cursor" "pointer"
         , style "transition" "all 0.15s ease"
         , style "display" "flex"
         , style "justify-content" "space-between"
         , style "align-items" "center"
         , style "border-top" ("1px solid " ++ UI.colors.border)
         , style "text-decoration" "none"
         , style "color" "inherit"
         ]
            ++ UI.zebraOverlay index
        )
        [ UI.row "0.75rem"
            [ div
                [ style "width" "20px"
                , style "height" "20px"
                , style "background-color" UI.colors.accentDim
                , style "border-radius" "2px"
                , style "display" "flex"
                , style "align-items" "center"
                , style "justify-content" "center"
                , style "color" UI.colors.accent
                , style "font-family" UI.fontMono
                , style "font-size" "0.75rem"
                , style "font-weight" "700"
                ]
                [ text topicIcon ]
            , div []
                [ div
                    [ style "font-size" "0.8125rem"
                    , style "color" UI.colors.textPrimary
                    , style "font-weight" "500"
                    ]
                    [ text topicLabel ]
                , div
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.6875rem"
                    , style "color" UI.colors.textMuted
                    , style "letter-spacing" "0.05em"
                    ]
                    [ text (String.fromInt chat.messageCount ++ " messages") ]
                ]
            ]
        , div
            [ style "width" "24px"
            , style "height" "24px"
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "2px"
            , style "display" "flex"
            , style "align-items" "center"
            , style "justify-content" "center"
            , style "color" UI.colors.textMuted
            , style "font-size" "0.75rem"
            ]
            [ text "→" ]
        ]


orElse : Maybe a -> Maybe a -> Maybe a
orElse fallback primary =
    case primary of
        Just _ ->
            primary

        Nothing ->
            fallback


viewMessageList : String -> RemoteData MessagesPage -> String -> (Int -> msg) -> (String -> msg) -> msg -> Html msg
viewMessageList chatId messagesPage searchQuery onPageChange onSearchChange onSearchSubmit =
    div []
        [ viewSearchBar searchQuery onSearchChange onSearchSubmit
        , case messagesPage of
            Loading ->
                UI.loadingSpinner

            Failure err ->
                UI.card [] [ text ("Error: " ++ err) ]

            NotAsked ->
                UI.loadingText "Loading messages..."

            Success page ->
                let
                    paginationConfig =
                        { page = page.page
                        , totalPages = page.totalPages
                        , onPageChange = onPageChange
                        }
                in
                div []
                    [ -- Top bar with page info and compact pagination
                      div
                        [ style "display" "flex"
                        , style "justify-content" "space-between"
                        , style "align-items" "center"
                        , style "flex-wrap" "wrap"
                        , style "gap" "0.75rem"
                        , style "margin-bottom" "1rem"
                        ]
                        [ UI.pageInfo
                            { page = page.page
                            , pageSize = page.pageSize
                            , total = page.total
                            , totalPages = page.totalPages
                            }
                        , if page.totalPages > 1 then
                            Components.Pagination.viewCompact paginationConfig
                          else
                            text ""
                        ]
                    , if List.isEmpty page.messages then
                        UI.card []
                            [ UI.emptyStateWithIcon "—"
                                (if String.isEmpty searchQuery then
                                    "No messages in this chat"
                                 else
                                    "No messages matching \"" ++ searchQuery ++ "\""
                                )
                            ]
                      else
                        UI.col "1px" (List.indexedMap (viewMessage chatId) page.messages)
                    , if page.totalPages > 1 then
                        Components.Pagination.view paginationConfig
                      else
                        text ""
                    ]
        ]


viewSearchBar : String -> (String -> msg) -> msg -> Html msg
viewSearchBar searchQuery onSearchChange onSearchSubmit =
    Html.form
        [ Html.Events.onSubmit onSearchSubmit
        , style "display" "flex"
        , style "gap" "0.5rem"
        , style "margin-bottom" "1rem"
        ]
        [ input
            [ type_ "text"
            , placeholder "Search messages..."
            , value searchQuery
            , onInput onSearchChange
            , style "flex" "1"
            , style "padding" "0.625rem 0.875rem"
            , style "background-color" UI.colors.bgTertiary
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "4px"
            , style "color" UI.colors.textPrimary
            , style "font-family" UI.fontMono
            , style "font-size" "0.8125rem"
            ]
            []
        , button
            [ type_ "submit"
            , style "padding" "0.625rem 1rem"
            , style "background-color" UI.colors.accentDim
            , style "border" "none"
            , style "border-radius" "4px"
            , style "color" UI.colors.accent
            , style "font-family" UI.fontMono
            , style "font-size" "0.75rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.05em"
            , style "cursor" "pointer"
            ]
            [ text "SEARCH" ]
        ]


viewMessage : String -> Int -> StoredMessage -> Html msg
viewMessage chatId index msg =
    let
        isInbound = msg.direction == "inbound"
        accentColor = if isInbound then UI.colors.accent else UI.colors.success
        roleLabel = if isInbound then "USER" else "BOT"

        maybeBadge =
            msg.mediaType
                |> Maybe.map
                    (\mediaType ->
                        case mediaType of
                            "voice" -> ("◉", "Voice")
                            "video" -> ("▶", "Video")
                            "video_note" -> ("●", "Video Note")
                            "photo" -> ("◫", "Photo")
                            "animation" -> ("∞", "GIF")
                            "document" -> ("◧", "Document")
                            other -> ("◇", other)
                    )
    in
    div
        ([ style "background-color" UI.colors.bgTertiary
         , style "border-left" ("3px solid " ++ accentColor)
         , style "padding" "1rem 1.25rem"
         ]
            ++ UI.zebraOverlay index
        )
        [ -- Header row
          div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "justify-content" "space-between"
            , style "margin-bottom" "0.75rem"
            , style "flex-wrap" "wrap"
            , style "gap" "0.5rem"
            ]
            [ UI.row "0.75rem"
                [ UI.roleBadge roleLabel accentColor
                , case maybeBadge of
                    Just (icon, labelText) ->
                        UI.mediaTypeBadge icon labelText

                    Nothing ->
                        if isInbound then
                            UI.mediaTypeBadge "" "TEXT"
                        else
                            text ""
                ]
            , UI.timestamp msg.timestamp
            ]
        , -- Media content
          viewMediaContent chatId msg
        , -- Message text
          Components.Markdown.view msg.content
        ]


viewMediaContent : String -> StoredMessage -> Html msg
viewMediaContent chatId msg =
    case ( msg.mediaType, msg.mediaPath ) of
        ( Just mediaType, Just path ) ->
            case filenameFromPath path of
                Just filename ->
                    let
                        url = mediaUrl chatId filename
                    in
                    div [ style "margin-bottom" "1rem" ]
                        [ viewMediaElement mediaType url filename ]

                Nothing ->
                    text ""

        _ ->
            text ""


viewMediaElement : String -> String -> String -> Html msg
viewMediaElement mediaType url filename =
    case mediaType of
        "voice" ->
            audio
                [ src url
                , controls True
                , style "width" "100%"
                , style "height" "40px"
                , style "border-radius" "4px"
                ]
                []

        "video" ->
            div
                [ style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "4px"
                , style "overflow" "hidden"
                , style "background-color" UI.colors.bgSurface
                ]
                [ video
                    [ src url
                    , controls True
                    , style "max-height" "400px"
                    , style "width" "100%"
                    , style "display" "block"
                    ]
                    []
                ]

        "video_note" ->
            div [ style "display" "flex", style "justify-content" "flex-start" ]
                [ div
                    [ style "border" ("2px solid " ++ UI.colors.accent)
                    , style "border-radius" "50%"
                    , style "overflow" "hidden"
                    , style "box-shadow" ("0 0 20px " ++ UI.colors.accentGlow)
                    ]
                    [ video
                        [ src url
                        , controls True
                        , style "width" "200px"
                        , style "height" "200px"
                        , style "object-fit" "cover"
                        , style "display" "block"
                        ]
                        []
                    ]
                ]

        "photo" ->
            div
                [ style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "4px"
                , style "overflow" "hidden"
                , style "background-color" UI.colors.bgSurface
                ]
                [ img
                    [ src url
                    , alt "Photo"
                    , style "max-height" "400px"
                    , style "width" "100%"
                    , style "object-fit" "contain"
                    , style "display" "block"
                    ]
                    []
                ]

        "animation" ->
            div
                [ style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "4px"
                , style "overflow" "hidden"
                ]
                [ video
                    [ src url
                    , autoplay True
                    , loop True
                    , attribute "muted" "true"
                    , style "max-height" "300px"
                    , style "width" "100%"
                    , style "display" "block"
                    ]
                    []
                ]

        "document" ->
            a
                [ href url
                , target "_blank"
                , style "display" "flex"
                , style "align-items" "center"
                , style "gap" "1rem"
                , style "padding" "1rem"
                , style "background-color" UI.colors.bgSurface
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "4px"
                , style "color" UI.colors.textPrimary
                , style "text-decoration" "none"
                , style "transition" "all 0.15s ease"
                ]
                [ div
                    [ style "width" "40px"
                    , style "height" "40px"
                    , style "background-color" UI.colors.accentDim
                    , style "border-radius" "2px"
                    , style "display" "flex"
                    , style "align-items" "center"
                    , style "justify-content" "center"
                    , style "color" UI.colors.accent
                    , style "font-size" "1.25rem"
                    ]
                    [ text "◧" ]
                , div []
                    [ div [ style "font-weight" "500", style "font-size" "0.875rem" ]
                        [ text filename ]
                    , div
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.6875rem"
                        , style "color" UI.colors.textMuted
                        , style "margin-top" "0.125rem"
                        , style "letter-spacing" "0.05em"
                        ]
                        [ text "CLICK TO DOWNLOAD" ]
                    ]
                ]

        _ ->
            a
                [ href url
                , target "_blank"
                , style "display" "inline-flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "padding" "0.75rem 1rem"
                , style "background-color" UI.colors.bgSurface
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "4px"
                , style "color" UI.colors.accent
                , style "text-decoration" "none"
                , style "font-family" UI.fontMono
                , style "font-size" "0.8125rem"
                ]
                [ text ("◇ " ++ filename) ]
