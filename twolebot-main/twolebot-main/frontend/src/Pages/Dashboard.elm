module Pages.Dashboard exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Encode
import Time
import Types exposing (..)
import UI


view : RemoteData FeedData -> RemoteData ResponseFeedData -> RemoteData SemanticStatus -> RemoteData TunnelStatus -> Time.Posix -> msg -> (Bool -> msg) -> msg -> Html msg
view feedData responsesData semanticStatus tunnelStatus currentTime onRefresh onToggleSemantic onReindex =
    div []
        [ UI.pageHeader "Dashboard"
            [ UI.button_ [ onClick onRefresh ] "Refresh" ]
        , UI.col "2rem"
            [ viewTunnel tunnelStatus
            , viewSemanticIndexer semanticStatus currentTime onToggleSemantic onReindex
            , case feedData of
                Loading ->
                    UI.loadingSpinner

                Failure err ->
                    UI.card [] [ text ("Error: " ++ err) ]

                NotAsked ->
                    UI.loadingText "Loading dashboard..."

                Success feed ->
                    UI.col "2rem"
                        [ viewStatsRow feed responsesData
                        , UI.gridTwo
                            [ viewCurrentlyProcessing feed.running
                            , viewResponseQueue responsesData
                            ]
                        , viewPendingQueue feed.pendingCount feed.pending
                        , viewRecentCompleted feed.completedCount feed.recentCompleted
                        ]
            ]
        ]


viewStatsRow : FeedData -> RemoteData ResponseFeedData -> Html msg
viewStatsRow feed responsesData =
    let
        ( responsesSent, responsesFailed ) =
            case responsesData of
                Success responses ->
                    ( responses.sentCount
                    , responses.failedCount
                    )
                _ ->
                    (0, 0)
    in
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fit, minmax(200px, 1fr))"
        , style "gap" "1rem"
        ]
        [ UI.statCard "Pending" (String.fromInt feed.pendingCount) UI.colors.warning
        , UI.statCard "Completed" (String.fromInt feed.completedCount) UI.colors.success
        , UI.statCard "Responses" (String.fromInt responsesSent) UI.colors.accent
        , UI.statCard "Failed" (String.fromInt responsesFailed) UI.colors.error
        ]


viewCurrentlyProcessing : Maybe PromptItem -> Html msg
viewCurrentlyProcessing maybePrompt =
    UI.cardWithHeader "Currently Processing" []
        [ case maybePrompt of
            Nothing ->
                viewIdleState

            Just prompt ->
                viewProcessingState prompt
        ]


viewIdleState : Html msg
viewIdleState =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" "1rem"
        , style "padding" "1.25rem"
        , style "background-color" UI.colors.successDim
        , style "border" ("1px solid " ++ UI.colors.success)
        , style "border-left" ("3px solid " ++ UI.colors.success)
        , style "border-radius" "4px"
        ]
        [ UI.statusDot UI.colors.success True
        , div []
            [ div
                [ style "font-family" UI.fontMono
                , style "font-size" "0.75rem"
                , style "font-weight" "600"
                , style "letter-spacing" "0.1em"
                , style "color" UI.colors.success
                ]
                [ text "IDLE" ]
            , div
                [ style "font-size" "0.8125rem"
                , style "color" UI.colors.textSecondary
                , style "margin-top" "0.25rem"
                ]
                [ text "Ready for requests" ]
            ]
        ]


viewProcessingState : PromptItem -> Html msg
viewProcessingState prompt =
    div
        [ style "background-color" UI.colors.warningDim
        , style "border" ("1px solid " ++ UI.colors.warning)
        , style "border-left" ("3px solid " ++ UI.colors.warning)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        ]
        [ UI.rowBetween
            [ UI.row "1rem"
                [ UI.statusDot UI.colors.warning True
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-weight" "600"
                    , style "font-size" "0.75rem"
                    , style "letter-spacing" "0.1em"
                    , style "color" UI.colors.warning
                    ]
                    [ text "PROCESSING" ]
                ]
            , span
                [ style "font-family" UI.fontMono
                , style "color" UI.colors.textMuted
                , style "font-size" "0.6875rem"
                , style "letter-spacing" "0.05em"
                ]
                [ text (prompt.sourceType ++ " USER " ++ String.fromInt prompt.userId) ]
            ]
        , div
            [ style "color" UI.colors.textSecondary
            , style "font-size" "0.9375rem"
            , style "line-height" "1.6"
            , style "white-space" "pre-wrap"
            , style "margin" "1rem 0"
            ]
            [ text (UI.truncateText 200 prompt.prompt) ]
        , div
            [ style "font-family" UI.fontMono
            , style "font-size" "0.6875rem"
            , style "color" UI.colors.textMuted
            , style "letter-spacing" "0.05em"
            ]
            [ text ("STARTED " ++ UI.formatDateTime prompt.createdAt) ]
        ]


-- ==================== Tunnel ====================


viewTunnel : RemoteData TunnelStatus -> Html msg
viewTunnel tunnelStatus =
    case tunnelStatus of
        Success status ->
            if status.active then
                viewTunnelActive status

            else
                text ""

        Failure _ ->
            -- Tunnel endpoint not available (e.g. --no-tunnel) — hide silently
            text ""

        _ ->
            text ""


viewTunnelActive : TunnelStatus -> Html msg
viewTunnelActive status =
    UI.cardWithHeader "Tunnel" []
        [ div
            [ style "display" "flex"
            , style "align-items" "flex-start"
            , style "gap" "1.5rem"
            , style "flex-wrap" "wrap"
            ]
            [ -- QR code (web component handles innerHTML safely outside Elm's VDOM)
              case status.qrSvg of
                Just svg ->
                    Html.node "qr-svg"
                        [ style "display" "block"
                        , style "flex-shrink" "0"
                        , style "width" "160px"
                        , style "height" "160px"
                        , Html.Attributes.property "content" (Json.Encode.string svg)
                        ]
                        []

                Nothing ->
                    text ""
            , -- Info
              div [ style "flex" "1", style "min-width" "200px" ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    , style "margin-bottom" "0.75rem"
                    ]
                    [ UI.statusDot UI.colors.success True
                    , span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.75rem"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.1em"
                        , style "color" UI.colors.success
                        ]
                        [ text "ACTIVE" ]
                    ]
                , case status.url of
                    Just url ->
                        div []
                            [ div
                                [ style "margin-bottom" "0.5rem" ]
                                [ a
                                    [ href (url ++ "/chat")
                                    , Html.Attributes.target "_blank"
                                    , style "color" UI.colors.accent
                                    , style "font-family" UI.fontMono
                                    , style "font-size" "0.8125rem"
                                    , style "text-decoration" "none"
                                    ]
                                    [ text (url ++ "/chat") ]
                                ]
                            , div
                                [ style "font-size" "0.8125rem"
                                , style "color" UI.colors.textSecondary
                                , style "line-height" "1.5"
                                ]
                                [ text "Scan QR to log in from your phone" ]
                            ]

                    Nothing ->
                        text ""
                ]
            ]
        ]


-- ==================== Semantic Indexer ====================


viewSemanticIndexer : RemoteData SemanticStatus -> Time.Posix -> (Bool -> msg) -> msg -> Html msg
viewSemanticIndexer semanticStatus currentTime onToggle onReindex =
    UI.cardWithHeader "Semantic Indexer" []
        [ case semanticStatus of
            NotAsked ->
                text ""

            Loading ->
                UI.loadingSpinner

            Failure _ ->
                viewSemanticCliDisabled

            Success status ->
                viewSemanticBody status currentTime onToggle onReindex
        ]


viewSemanticCliDisabled : Html msg
viewSemanticCliDisabled =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.75rem"
        , style "padding" "1rem"
        , style "background-color" UI.colors.bgSurface
        , style "border-radius" "4px"
        ]
        [ span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.75rem"
            , style "color" UI.colors.textMuted
            , style "letter-spacing" "0.05em"
            ]
            [ text "DISABLED" ]
        , span
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            ]
            [ text "Enable with --semantic flag" ]
        ]


{-| Unified view that always shows stats, regardless of paused state.
-}
viewSemanticBody : SemanticStatus -> Time.Posix -> (Bool -> msg) -> msg -> Html msg
viewSemanticBody status currentTime onToggle onReindex =
    let
        isPaused =
            not status.enabled
    in
    div []
        [ -- Control bar: pause/activate button + status
          div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "justify-content" "space-between"
            , style "margin-bottom" "1rem"
            ]
            [ if isPaused then
                div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    ]
                    [ UI.statusDot UI.colors.textMuted False
                    , span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.75rem"
                        , style "color" UI.colors.textMuted
                        , style "letter-spacing" "0.05em"
                        ]
                        [ text "PAUSED" ]
                    ]
              else
                viewNextPollCountdown status.lastConversationPollAt status.conversationPollIntervalSecs currentTime
            , statusActionButton isPaused onToggle
            ]
        -- Data source sections
        , viewDataSourceSection "Memory"
            status.memory
            status.totalMemoryFiles
            status.totalMemoryChunks
            status.totalMemoryFilesAvailable
            status.memoryFilesStale
            isPaused
            Nothing
        , div [ style "height" "0.75rem" ] []
        , viewDataSourceSection "Conversations"
            status.conversations
            status.totalConversationSessions
            status.totalConversationChunks
            status.totalConversationFilesAvailable
            status.conversationFilesStale
            isPaused
            (if isPaused then Nothing else Just onReindex)
        ]


{-| A data source section showing task status, file counts, and stale info.
-}
viewDataSourceSection : String -> TaskStatus -> Int -> Int -> Int -> Int -> Bool -> Maybe msg -> Html msg
viewDataSourceSection name task filesIndexed chunksCount totalAvailable staleCount isPaused maybeReindex =
    let
        ( statusColor, statusLabel, isAnimated ) =
            if isPaused then
                ( UI.colors.textMuted, "PAUSED", False )
            else
                case task.activity of
                    "idle" ->
                        ( UI.colors.success, "IDLE", False )

                    "initial_index" ->
                        ( UI.colors.warning, "INDEXING", True )

                    "indexing" ->
                        ( UI.colors.warning, "INDEXING", True )

                    "polling" ->
                        ( UI.colors.accent, "POLLING", True )

                    _ ->
                        ( UI.colors.textMuted, String.toUpper task.activity, False )
    in
    div
        [ style "padding" "0.75rem"
        , style "background-color" UI.colors.bgSurface
        , style "border-radius" "4px"
        , style "border-left" ("3px solid " ++ statusColor)
        ]
        [ -- Header: name + status + optional Run Now
          div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "justify-content" "space-between"
            , style "margin-bottom" "0.5rem"
            ]
            [ div
                [ style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                ]
                [ UI.statusDot statusColor isAnimated
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.6875rem"
                    , style "font-weight" "600"
                    , style "letter-spacing" "0.05em"
                    , style "color" UI.colors.textPrimary
                    ]
                    [ text name ]
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "color" statusColor
                    , style "letter-spacing" "0.05em"
                    ]
                    [ text statusLabel ]
                ]
            , case maybeReindex of
                Just reindexMsg ->
                    let
                        isRunning =
                            task.activity == "polling" || task.activity == "indexing" || task.activity == "initial_index"

                        ( btnLabel, btnColor, btnCursor ) =
                            if isRunning then
                                ( "Indexing...", UI.colors.warning, "wait" )
                            else
                                ( "Run Now", UI.colors.accent, "pointer" )
                    in
                    button
                        [ onClick reindexMsg
                        , disabled isRunning
                        , style "display" "flex"
                        , style "align-items" "center"
                        , style "gap" "0.375rem"
                        , style "background-color" "transparent"
                        , style "color" btnColor
                        , style "border" ("1px solid " ++ btnColor)
                        , style "padding" "0.25rem 0.625rem"
                        , style "border-radius" "2px"
                        , style "cursor" btnCursor
                        , style "font-family" UI.fontMono
                        , style "font-size" "0.625rem"
                        , style "font-weight" "500"
                        , style "letter-spacing" "0.05em"
                        , style "text-transform" "uppercase"
                        , style "transition" "all 0.15s ease"
                        , style "flex-shrink" "0"
                        , style "opacity" (if isRunning then "0.7" else "1")
                        ]
                        [ text btnLabel ]

                Nothing ->
                    text ""
            ]
        -- File counts: indexed / stale / total
        , div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "gap" "1rem"
            , style "font-family" UI.fontMono
            , style "font-size" "0.6875rem"
            ]
            [ viewCountBadge (String.fromInt filesIndexed ++ " indexed") UI.colors.accent
            , if staleCount > 0 then
                viewCountBadge (String.fromInt staleCount ++ " stale") UI.colors.warning
              else
                text ""
            , span
                [ style "color" UI.colors.textMuted ]
                [ text (String.fromInt totalAvailable ++ " files on disk") ]
            , span
                [ style "color" UI.colors.textMuted
                , style "font-size" "0.625rem"
                ]
                [ text (String.fromInt chunksCount ++ " chunks") ]
            ]
        -- Active progress (when indexing)
        , case task.filesTotal of
            Just total ->
                div
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "color" UI.colors.textSecondary
                    , style "margin-top" "0.375rem"
                    ]
                    [ text ("Progress: " ++ String.fromInt (task.filesIndexed + task.filesSkipped) ++ "/" ++ String.fromInt total) ]

            Nothing ->
                text ""
        , case task.currentFile of
            Just file ->
                div
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "color" UI.colors.textMuted
                    , style "margin-top" "0.25rem"
                    , style "white-space" "nowrap"
                    , style "overflow" "hidden"
                    , style "text-overflow" "ellipsis"
                    ]
                    [ text file ]

            Nothing ->
                text ""
        ]


viewCountBadge : String -> String -> Html msg
viewCountBadge label color =
    span
        [ style "color" color
        , style "font-weight" "500"
        ]
        [ text label ]


viewPendingQueue : Int -> List PromptItem -> Html msg
viewPendingQueue count prompts =
    UI.cardWithHeader ("Pending Queue (" ++ String.fromInt count ++ ")") []
        [ if List.isEmpty prompts then
            UI.emptyStateWithIcon "—" "No pending prompts"
          else
            UI.col "0.5rem" (List.indexedMap viewPromptItemCompact prompts)
        ]


viewResponseQueue : RemoteData ResponseFeedData -> Html msg
viewResponseQueue responsesData =
    UI.cardWithHeader "Response Queue" []
        [ case responsesData of
            Success responses ->
                div []
                    [ div
                        [ style "display" "grid"
                        , style "grid-template-columns" "repeat(auto-fit, minmax(140px, 1fr))"
                        , style "gap" "1rem"
                        , style "text-align" "center"
                        ]
                        [ UI.miniStat "Pending" responses.pendingCount UI.colors.warning
                        , UI.miniStat "Sent" responses.sentCount UI.colors.success
                        , UI.miniStat "Failed" responses.failedCount UI.colors.error
                        ]
                    , if not (List.isEmpty responses.recentFailed) then
                        div
                            [ style "margin-top" "1.25rem"
                            , style "padding-top" "1.25rem"
                            , style "border-top" ("1px solid " ++ UI.colors.border)
                            ]
                            [ div
                                [ style "font-family" UI.fontMono
                                , style "font-size" "0.625rem"
                                , style "font-weight" "600"
                                , style "letter-spacing" "0.1em"
                                , style "color" UI.colors.error
                                , style "margin-bottom" "0.75rem"
                                ]
                                [ text "RECENT FAILURES" ]
                            , UI.col "0.375rem"
                                (List.take 3 responses.recentFailed |> List.map viewFailedResponse)
                            ]
                      else
                        text ""
                    ]

            _ ->
                UI.loadingSpinner
        ]


viewFailedResponse : ResponseItem -> Html msg
viewFailedResponse response =
    div
        [ style "font-family" UI.fontMono
        , style "font-size" "0.75rem"
        , style "color" UI.colors.textSecondary
        , style "padding" "0.5rem 0.75rem"
        , style "background-color" UI.colors.errorDim
        , style "border-left" ("2px solid " ++ UI.colors.error)
        , style "border-radius" "2px"
        ]
        [ text (UI.truncateText 50 response.content) ]


viewRecentCompleted : Int -> List PromptItem -> Html msg
viewRecentCompleted count prompts =
    UI.cardWithHeader ("Total Completed (" ++ String.fromInt count ++ ")") []
        [ if List.isEmpty prompts then
            UI.emptyStateWithIcon "—" "No completed prompts yet"
          else
            UI.col "0.5rem" (List.take 10 prompts |> List.indexedMap viewPromptItemCompact)
        ]


viewPromptItemCompact : Int -> PromptItem -> Html msg
viewPromptItemCompact index prompt =
    let
        borderColor =
            case prompt.status of
                "completed" -> UI.colors.success
                "running" -> UI.colors.warning
                "failed" -> UI.colors.error
                _ -> UI.colors.textMuted
    in
    UI.accentedItem borderColor (UI.zebraOverlay index)
        [ div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "justify-content" "space-between"
            , style "margin-bottom" "0.5rem"
            , style "flex-wrap" "wrap"
            , style "gap" "0.5rem"
            ]
            [ UI.row "0.75rem"
                [ UI.statusBadge prompt.status
                , span
                    [ style "font-family" UI.fontMono
                    , style "color" UI.colors.textSecondary
                    , style "font-size" "0.75rem"
                    , style "letter-spacing" "0.02em"
                    ]
                    [ text (prompt.sourceType ++ " User " ++ String.fromInt prompt.userId) ]
                ]
            , UI.row "0.5rem"
                [ UI.timestamp prompt.createdAt
                , case prompt.completedAt of
                    Just completed ->
                        span
                            [ style "font-family" UI.fontMono
                            , style "color" UI.colors.textMuted
                            , style "font-size" "0.6875rem"
                            ]
                            [ text (" → " ++ UI.formatDateTime completed) ]
                    Nothing ->
                        text ""
                ]
            ]
        , div
            [ style "color" UI.colors.textSecondary
            , style "font-size" "0.875rem"
            , style "line-height" "1.5"
            ]
            [ text (UI.truncateText 100 prompt.prompt) ]
        ]


viewNextPollCountdown : Maybe Int -> Int -> Time.Posix -> Html msg
viewNextPollCountdown maybeLastPoll intervalSecs currentTime =
    let
        currentSeconds =
            Time.posixToMillis currentTime // 1000

        ( label, color ) =
            case maybeLastPoll of
                Nothing ->
                    ( "Waiting for first poll...", UI.colors.textMuted )

                Just lastPollSeconds ->
                    let
                        nextPollSeconds =
                            lastPollSeconds + intervalSecs

                        remaining =
                            Basics.max 0 (nextPollSeconds - currentSeconds)
                    in
                    if remaining == 0 then
                        ( "Polling now...", UI.colors.warning )

                    else if remaining < 60 then
                        ( String.fromInt remaining ++ "s until next poll", UI.colors.textSecondary )

                    else
                        let
                            mins =
                                remaining // 60

                            secs =
                                modBy 60 remaining
                        in
                        ( String.fromInt mins ++ "m " ++ String.fromInt secs ++ "s until next poll"
                        , UI.colors.textSecondary
                        )
    in
    span
        [ style "font-family" UI.fontMono
        , style "font-size" "0.6875rem"
        , style "color" color
        , style "letter-spacing" "0.02em"
        ]
        [ text label ]


{-| Button that shows current state and what clicking will do.
    activate=True means "click to activate" (currently paused).
    activate=False means "click to pause" (currently active).
-}
statusActionButton : Bool -> (Bool -> msg) -> Html msg
statusActionButton activate onToggle =
    let
        ( label, dotColor, borderColor ) =
            if activate then
                ( "Activate", UI.colors.success, UI.colors.success )

            else
                ( "Pause", UI.colors.warning, UI.colors.border )
    in
    button
        [ onClick (onToggle activate)
        , style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.5rem"
        , style "background-color" "transparent"
        , style "color" UI.colors.textSecondary
        , style "border" ("1px solid " ++ borderColor)
        , style "padding" "0.375rem 0.875rem"
        , style "border-radius" "2px"
        , style "cursor" "pointer"
        , style "font-family" UI.fontMono
        , style "font-size" "0.6875rem"
        , style "font-weight" "500"
        , style "letter-spacing" "0.05em"
        , style "text-transform" "uppercase"
        , style "transition" "all 0.15s ease"
        , style "flex-shrink" "0"
        ]
        [ span
            [ style "width" "8px"
            , style "height" "8px"
            , style "border-radius" "50%"
            , style "background-color" dotColor
            , style "flex-shrink" "0"
            ]
            []
        , text label
        ]
