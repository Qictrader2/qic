module Pages.LiveBoard exposing (view)

import Components.TaskCard
import Dict exposing (Dict)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as D
import Types exposing (..)
import UI


view :
    RemoteData LiveBoard
    -> RemoteData (List WorkProject)
    -> Bool
    -> TaskViewMode
    -> Maybe Int
    -> Maybe ( Int, String )
    -> String
    -> { onNavigate : Route -> msg
       , onRefresh : msg
       , onSelectTask : Int -> msg
       , onDeselectTask : Int -> msg
       , onClearCompleted : msg
       , onMoveSelection : Int -> String -> msg
       , onEnsureAgent : msg
       , onStopAgent : msg
       , onToggleViewMode : msg
       , onDragStart : Int -> msg
       , onDragEnd : msg
       , onDragOver : msg
       , onDropOnStatus : String -> msg
       , onBoardDropCommentChange : String -> msg
       , onSubmitBoardDrop : msg
       , onCancelBoardDrop : msg
       }
    -> Html msg
view liveBoard projects isBusy viewMode draggingTaskId boardDropTarget boardDropComment config =
    let
        projectDict =
            case projects of
                Success projectList ->
                    List.foldl (\p acc -> Dict.insert p.id p.name acc) Dict.empty projectList

                _ ->
                    Dict.empty

        agentButton =
            case liveBoard of
                Success board ->
                    if board.stats.agentLoopState == "running" then
                        UI.button_ [ onClick config.onStopAgent, disabled isBusy ] (if isBusy then "Stopping..." else "Pause Agent")
                    else
                        UI.primaryButton [ onClick config.onEnsureAgent, disabled isBusy ] (if isBusy then "Starting..." else "Start Autowork")

                _ ->
                    text ""
    in
    div []
        [ UI.pageHeader "Live Board"
            [ UI.button_
                [ onClick config.onToggleViewMode
                , style "padding" "0.375rem 0.75rem"
                , style "font-size" "0.625rem"
                ]
                (case viewMode of
                    ListView -> "Board View"
                    BoardView -> "List View"
                )
            , UI.button_ [ onClick config.onRefresh, title "Refresh Live Board", disabled isBusy ] "Refresh"
            , agentButton
            ]
        , case liveBoard of
            NotAsked ->
                UI.emptyState "Loading..."

            Loading ->
                UI.loadingSpinner

            Failure err ->
                UI.emptyState ("Error: " ++ err)

            Success board ->
                div []
                    [ viewStats board.stats
                    , -- Board drop comment modal
                      case boardDropTarget of
                        Just ( _, _ ) ->
                            boardDropCommentModal boardDropComment isBusy config

                        Nothing ->
                            text ""
                    , case viewMode of
                        BoardView ->
                            viewBoardMode board projectDict draggingTaskId config

                        ListView ->
                            viewListMode board projectDict isBusy config
                    ]
        ]


viewStats : LiveBoardStats -> Html msg
viewStats stats =
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fit, minmax(120px, 1fr))"
        , style "gap" "0.75rem"
        , style "margin-bottom" "1.5rem"
        ]
        [ UI.statCard "Backlog" (String.fromInt stats.totalBacklog) UI.colors.textSecondary
        , UI.statCard "Selected" (String.fromInt stats.totalSelected) UI.colors.accent
        , UI.statCard "Queued" (String.fromInt stats.queued) UI.colors.warning
        , UI.statCard "Completed" (String.fromInt stats.completed) UI.colors.success
        , UI.statCard "Failed" (String.fromInt stats.failed) UI.colors.error
        , agentStateCard stats.agentLoopState stats.active
        ]


agentStateCard : String -> Maybe Int -> Html msg
agentStateCard state active =
    let
        ( color, label, shouldPulse ) =
            case state of
                "running" -> ( UI.colors.success, "RUNNING", True )
                "paused" -> ( UI.colors.warning, "PAUSED", False )
                _ -> ( UI.colors.textMuted, "IDLE", False )
    in
    div
        [ style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        ]
        [ div
            [ style "font-family" UI.fontMono
            , style "font-size" "0.625rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.12em"
            , style "text-transform" "uppercase"
            , style "color" UI.colors.textMuted
            , style "margin-bottom" "0.625rem"
            ]
            [ text "Agent Loop" ]
        , div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.5rem"
            ]
            [ UI.statusDot color shouldPulse
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" color
                ]
                [ text label ]
            ]
        , case active of
            Just taskId ->
                div
                    [ style "margin-top" "0.5rem"
                    , style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "color" UI.colors.textSecondary
                    ]
                    [ text ("Active: #" ++ String.fromInt taskId) ]

            Nothing ->
                text ""
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- BOARD VIEW (Kanban columns by task status)
-- ═══════════════════════════════════════════════════════════════════════════


viewBoardMode :
    LiveBoard
    -> Dict Int String
    -> Maybe Int
    -> { a | onNavigate : Route -> msg, onDragStart : Int -> msg, onDragEnd : msg, onDragOver : msg, onDropOnStatus : String -> msg }
    -> Html msg
viewBoardMode board projectDict draggingTaskId config =
    let
        -- Build a set of selected task IDs for badge display
        selectedIds =
            List.map (\st -> st.task.id) board.selected
                |> List.foldl (\id acc -> Dict.insert id True acc) Dict.empty

        -- Combine backlog (excluding already-selected) + selected tasks
        allTasks =
            List.filter (\t -> not (Dict.member t.id selectedIds)) board.backlog
                ++ List.map .task board.selected

        -- Build selection status lookup
        selectionStatuses =
            List.foldl (\st acc -> Dict.insert st.task.id st.selection.status acc) Dict.empty board.selected

        statuses =
            [ "todo", "in_progress", "ready_for_review", "under_review", "done", "blocked" ]

        tasksByStatus status =
            List.filter (\t -> t.status == status) allTasks
    in
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fill, minmax(200px, 1fr))"
        , style "gap" "0.75rem"
        , style "min-height" "200px"
        ]
        (List.map (\s -> liveBoardStatusColumn s (tasksByStatus s) draggingTaskId projectDict selectedIds selectionStatuses config) statuses)


liveBoardStatusColumn :
    String
    -> List WorkTask
    -> Maybe Int
    -> Dict Int String
    -> Dict Int Bool
    -> Dict Int String
    -> { a | onNavigate : Route -> msg, onDragStart : Int -> msg, onDragEnd : msg, onDragOver : msg, onDropOnStatus : String -> msg }
    -> Html msg
liveBoardStatusColumn status tasks draggingTaskId projectDict selectedIds selectionStatuses config =
    let
        isDragOver = draggingTaskId /= Nothing
    in
    div
        [ Html.Events.preventDefaultOn "dragover" (D.succeed ( config.onDragOver, True ))
        , Html.Events.preventDefaultOn "drop" (D.succeed ( config.onDropOnStatus status, True ))
        , style "background-color" (if isDragOver then UI.colors.bgTertiary else UI.colors.bgSurface)
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "0.75rem"
        , style "min-height" "150px"
        , style "transition" "background-color 0.15s ease"
        ]
        [ -- Column header
          div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.5rem"
            , style "margin-bottom" "0.75rem"
            , style "padding-bottom" "0.5rem"
            , style "border-bottom" ("2px solid " ++ UI.taskStatusColor status)
            ]
            [ span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.5625rem"
                , style "font-weight" "600"
                , style "letter-spacing" "0.1em"
                , style "text-transform" "uppercase"
                , style "color" (UI.taskStatusColor status)
                ]
                [ text (taskStatusLabel status) ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.5625rem"
                , style "color" UI.colors.textMuted
                ]
                [ text (String.fromInt (List.length tasks)) ]
            ]
        , -- Task cards
          div
            [ style "display" "flex"
            , style "flex-direction" "column"
            , style "gap" "0.375rem"
            ]
            (List.map (\t -> liveBoardTaskCard t projectDict selectedIds selectionStatuses config) tasks)
        ]


liveBoardTaskCard :
    WorkTask
    -> Dict Int String
    -> Dict Int Bool
    -> Dict Int String
    -> { a | onNavigate : Route -> msg, onDragStart : Int -> msg, onDragEnd : msg }
    -> Html msg
liveBoardTaskCard task projectDict selectedIds selectionStatuses config =
    let
        isSelected =
            Dict.member task.id selectedIds

        selectionStatus =
            Dict.get task.id selectionStatuses

        projectName =
            Dict.get task.projectId projectDict
                |> Maybe.withDefault ""
    in
    div
        [ attribute "draggable" "true"
        , on "dragstart" (D.succeed (config.onDragStart task.id))
        , on "dragend" (D.succeed config.onDragEnd)
        , onClick (config.onNavigate (TaskDetailRoute task.id))
        , style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-left" ("3px solid " ++ UI.taskStatusColor task.status)
        , style "border-radius" "3px"
        , style "padding" "0.5rem 0.625rem"
        , style "cursor" "grab"
        , style "font-size" "0.8125rem"
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "gap" "0.375rem"
            , style "margin-bottom" "0.25rem"
            ]
            [ span
                [ style "font-weight" "500"
                , style "color" UI.colors.textPrimary
                , style "overflow" "hidden"
                , style "text-overflow" "ellipsis"
                , style "white-space" "nowrap"
                ]
                [ text task.title ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.5625rem"
                , style "color" UI.colors.textMuted
                , style "white-space" "nowrap"
                ]
                [ text ("#" ++ String.fromInt task.id) ]
            ]
        , div
            [ style "display" "flex"
            , style "gap" "0.25rem"
            , style "flex-wrap" "wrap"
            , style "align-items" "center"
            ]
            ([ Components.TaskCard.taskPriorityBadge task.priority ]
                ++ (if not (String.isEmpty projectName) then
                        [ span
                            [ style "font-family" UI.fontMono
                            , style "font-size" "0.5rem"
                            , style "color" UI.colors.textMuted
                            ]
                            [ text projectName ]
                        ]
                    else
                        []
                   )
                ++ (if isSelected then
                        [ case selectionStatus of
                            Just ss -> selectionStatusBadge ss
                            Nothing -> text ""
                        ]
                    else
                        []
                   )
            )
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- LIST VIEW (original Selected + Backlog layout)
-- ═══════════════════════════════════════════════════════════════════════════


viewListMode :
    LiveBoard
    -> Dict Int String
    -> Bool
    -> { a
        | onNavigate : Route -> msg
        , onSelectTask : Int -> msg
        , onDeselectTask : Int -> msg
        , onClearCompleted : msg
        , onMoveSelection : Int -> String -> msg
       }
    -> Html msg
viewListMode board projectDict isBusy config =
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fit, minmax(300px, 1fr))"
        , style "gap" "1.5rem"
        ]
        [ viewSelectedQueue board.selected projectDict isBusy config.onNavigate config.onDeselectTask config.onClearCompleted config.onMoveSelection
        , viewBacklog board.backlog projectDict isBusy config.onNavigate config.onSelectTask
        ]


viewSelectedQueue :
    List SelectedTask
    -> Dict Int String
    -> Bool
    -> (Route -> msg)
    -> (Int -> msg)
    -> msg
    -> (Int -> String -> msg)
    -> Html msg
viewSelectedQueue selected projectDict isBusy onNavigate onDeselectTask onClearCompleted onMoveSelection =
    UI.card []
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-bottom" "1.25rem"
            , style "padding-bottom" "0.75rem"
            , style "border-bottom" ("1px solid " ++ UI.colors.border)
            ]
            [ div [ style "display" "flex", style "align-items" "center", style "gap" "0.625rem" ]
                [ div
                    [ style "width" "4px"
                    , style "height" "4px"
                    , style "background-color" UI.colors.accent
                    , style "box-shadow" ("0 0 6px " ++ UI.colors.accent)
                    ]
                    []
                , h3
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.6875rem"
                    , style "font-weight" "600"
                    , style "letter-spacing" "0.12em"
                    , style "text-transform" "uppercase"
                    , style "color" UI.colors.textSecondary
                    , style "margin" "0"
                    ]
                    [ text ("Selected (" ++ String.fromInt (List.length selected) ++ ")") ]
                ]
            , UI.button_ [ onClick onClearCompleted, disabled isBusy ] (if isBusy then "Working..." else "Clear Completed")
            ]
        , if List.isEmpty selected then
            UI.emptyState "No tasks selected. Select tasks from the backlog."
          else
            div
                [ style "display" "flex"
                , style "flex-direction" "column"
                , style "gap" "0.5rem"
                ]
                (List.map (\st -> viewSelectedTask st projectDict isBusy onNavigate onDeselectTask onMoveSelection)
                    (List.sortBy (\st -> st.selection.sortOrder) selected)
                )
        ]


viewSelectedTask : SelectedTask -> Dict Int String -> Bool -> (Route -> msg) -> (Int -> msg) -> (Int -> String -> msg) -> Html msg
viewSelectedTask st projectDict isBusy onNavigate onDeselectTask onMoveSelection =
    let
        selColor =
            case st.selection.status of
                "queued" -> UI.colors.warning
                "active" -> UI.colors.success
                "paused" -> UI.colors.textMuted
                "done" -> UI.colors.success
                "failed" -> UI.colors.error
                _ -> UI.colors.border

        projectName =
            Dict.get st.task.projectId projectDict
                |> Maybe.withDefault ("Project #" ++ String.fromInt st.task.projectId)
    in
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "0.75rem 1rem"
        , style "border-left" ("3px solid " ++ selColor)
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "flex-start"
            , style "gap" "0.5rem"
            ]
            [ div [ style "flex" "1", style "min-width" "0" ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    , style "margin-bottom" "0.375rem"
                    ]
                    [ span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.625rem"
                        , style "color" UI.colors.textMuted
                        ]
                        [ text ("#" ++ String.fromInt st.task.id) ]
                    , selectionStatusBadge st.selection.status
                    ]
                , a
                    [ href ("/tasks/" ++ String.fromInt st.task.id)
                    , style "display" "block"
                    , style "font-size" "0.875rem"
                    , style "font-weight" "500"
                    , style "color" UI.colors.textPrimary
                    , style "margin-bottom" "0.375rem"
                    , style "text-decoration" "none"
                    ]
                    [ text st.task.title ]
                , div
                    [ style "display" "flex"
                    , style "flex-wrap" "wrap"
                    , style "gap" "0.375rem"
                    , style "align-items" "center"
                    ]
                    [ Components.TaskCard.taskPriorityBadge st.task.priority
                    , span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.5625rem"
                        , style "color" UI.colors.textMuted
                        ]
                        [ text projectName ]
                    ]
                ]
            , div [ style "display" "flex", style "gap" "0.25rem" ]
                [ button
                    [ onClick (onMoveSelection st.task.id "top")
                    , disabled isBusy
                    , style "background" "transparent"
                    , style "border" ("1px solid " ++ UI.colors.border)
                    , style "color" UI.colors.textMuted
                    , style "padding" "0.25rem 0.5rem"
                    , style "border-radius" "2px"
                    , style "cursor" "pointer"
                    , style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    ]
                    [ text "Top" ]
                , button
                    [ onClick (onMoveSelection st.task.id "bottom")
                    , disabled isBusy
                    , style "background" "transparent"
                    , style "border" ("1px solid " ++ UI.colors.border)
                    , style "color" UI.colors.textMuted
                    , style "padding" "0.25rem 0.5rem"
                    , style "border-radius" "2px"
                    , style "cursor" "pointer"
                    , style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    ]
                    [ text "Bottom" ]
                , button
                    [ onClick (onDeselectTask st.task.id)
                    , disabled isBusy
                    , style "background" "transparent"
                    , style "border" ("1px solid " ++ UI.colors.border)
                    , style "color" UI.colors.textMuted
                    , style "padding" "0.25rem 0.5rem"
                    , style "border-radius" "2px"
                    , style "cursor" "pointer"
                    , style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    ]
                    [ text "Remove" ]
                ]
            ]
        ]


selectionStatusBadge : String -> Html msg
selectionStatusBadge status =
    let
        ( bgColor, textColor, label ) =
            case status of
                "queued" -> ( UI.colors.warningDim, UI.colors.warning, "QUEUED" )
                "active" -> ( UI.colors.successDim, UI.colors.success, "ACTIVE" )
                "paused" -> ( UI.colors.borderLight, UI.colors.textMuted, "PAUSED" )
                "done" -> ( UI.colors.successDim, UI.colors.success, "DONE" )
                "failed" -> ( UI.colors.errorDim, UI.colors.error, "FAILED" )
                _ -> ( UI.colors.borderLight, UI.colors.textMuted, String.toUpper status )
    in
    UI.pillBadge bgColor textColor label


viewBacklog : List WorkTask -> Dict Int String -> Bool -> (Route -> msg) -> (Int -> msg) -> Html msg
viewBacklog backlog projectDict isBusy onNavigate onSelectTask =
    UI.card []
        [ UI.cardHeader ("Backlog (" ++ String.fromInt (List.length backlog) ++ ")")
        , if List.isEmpty backlog then
            UI.emptyState "Backlog is empty."
          else
            div
                [ style "display" "flex"
                , style "flex-direction" "column"
                , style "gap" "0.5rem"
                ]
                (List.map (\t -> backlogItem t projectDict isBusy onNavigate onSelectTask) backlog)
        ]


backlogItem : WorkTask -> Dict Int String -> Bool -> (Route -> msg) -> (Int -> msg) -> Html msg
backlogItem task projectDict isBusy onNavigate onSelectTask =
    let
        projectName =
            Dict.get task.projectId projectDict
                |> Maybe.withDefault ("Project #" ++ String.fromInt task.projectId)
    in
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "3px"
        , style "padding" "0.75rem 1rem"
        , style "display" "flex"
        , style "justify-content" "space-between"
        , style "align-items" "center"
        , style "gap" "0.5rem"
        ]
        [ div
            [ style "flex" "1"
            , style "min-width" "0"
            ]
            [ div
                [ style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "margin-bottom" "0.25rem"
                ]
                [ a
                    [ href ("/tasks/" ++ String.fromInt task.id)
                    , style "font-size" "0.8125rem"
                    , style "font-weight" "500"
                    , style "color" UI.colors.textPrimary
                    , style "text-decoration" "none"
                    ]
                    [ text task.title ]
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    , style "color" UI.colors.textMuted
                    ]
                    [ text ("#" ++ String.fromInt task.id) ]
                ]
            , div
                [ style "display" "flex"
                , style "flex-wrap" "wrap"
                , style "gap" "0.375rem"
                , style "align-items" "center"
                ]
                [ Components.TaskCard.taskPriorityBadge task.priority
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    , style "color" UI.colors.textMuted
                    ]
                    [ text projectName ]
                ]
            ]
        , -- Select button
          UI.primaryButton
            [ onClick (onSelectTask task.id)
            , disabled isBusy
            , style "padding" "0.25rem 0.75rem"
            , style "font-size" "0.625rem"
            ]
            (if isBusy then "Selecting..." else "Select")
        ]



-- ═══════════════════════════════════════════════════════════════════════════
-- BOARD DROP COMMENT MODAL
-- ═══════════════════════════════════════════════════════════════════════════


boardDropCommentModal :
    String
    -> Bool
    -> { a | onBoardDropCommentChange : String -> msg, onSubmitBoardDrop : msg, onCancelBoardDrop : msg }
    -> Html msg
boardDropCommentModal commentText isBusy config =
    div
        [ style "position" "fixed"
        , style "top" "0"
        , style "left" "0"
        , style "right" "0"
        , style "bottom" "0"
        , style "background-color" "rgba(0,0,0,0.6)"
        , style "display" "flex"
        , style "align-items" "center"
        , style "justify-content" "center"
        , style "z-index" "200"
        ]
        [ div
            [ style "background-color" UI.colors.bgTertiary
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "4px"
            , style "padding" "1.5rem"
            , style "max-width" "480px"
            , style "width" "90%"
            ]
            [ UI.cardHeader "Ready for Review"
            , div
                [ style "font-family" UI.fontBody
                , style "font-size" "0.8125rem"
                , style "color" UI.colors.textSecondary
                , style "margin-bottom" "0.75rem"
                ]
                [ text "A comment is required when moving to ready for review." ]
            , textarea
                [ value commentText
                , onInput config.onBoardDropCommentChange
                , placeholder "What was completed? Key changes, testing done..."
                , disabled isBusy
                , style "width" "100%"
                , style "min-height" "80px"
                , style "background-color" UI.colors.bgPrimary
                , style "color" UI.colors.textPrimary
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "2px"
                , style "padding" "0.75rem"
                , style "font-family" UI.fontBody
                , style "font-size" "0.875rem"
                , style "resize" "vertical"
                , style "box-sizing" "border-box"
                ]
                []
            , div [ style "display" "flex", style "justify-content" "flex-end", style "gap" "0.5rem", style "margin-top" "0.75rem" ]
                [ UI.button_ [ onClick config.onCancelBoardDrop, disabled isBusy ] "Cancel"
                , UI.primaryButton [ onClick config.onSubmitBoardDrop, disabled (isBusy || String.isEmpty (String.trim commentText)) ] "Submit"
                ]
            ]
        ]
