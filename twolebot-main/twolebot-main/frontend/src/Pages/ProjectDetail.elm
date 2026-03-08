module Pages.ProjectDetail exposing (view)

import Components.Markdown
import Components.Tab
import Components.TaskCard
import Components.TaskFilters
import Components.Voice
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as D
import Types exposing (..)
import UI


view :
    RemoteData WorkProject
    -> RemoteData (List WorkTask)
    -> RemoteData (List WorkDocument)
    -> RemoteData (List ActivityLog)
    -> RemoteData TaskAnalytics
    -> ProjectTab
    -> Bool
    -> TaskForm
    -> Bool
    -> DocumentForm
    -> TaskFilters
    -> TaskViewMode
    -> Maybe Int
    -> Maybe ( Int, String )
    -> String
    -> Bool
    -> VoiceState
    -> { onNavigate : Route -> msg
       , onRefresh : msg
       , onTabChange : ProjectTab -> msg
       , onToggleTaskForm : msg
       , onTaskTitleChange : String -> msg
       , onTaskDescChange : String -> msg
       , onTaskPriorityChange : String -> msg
       , onSubmitTask : msg
       , onCloseTaskForm : msg
       , onToggleDocForm : msg
       , onDocTitleChange : String -> msg
       , onDocContentChange : String -> msg
       , onDocTypeChange : String -> msg
       , onSubmitDoc : msg
       , onCloseDocForm : msg
       , onToggleStatus : String -> msg
       , onClearFilters : msg
       , onTakeNextTask : msg
       , onTakeNextReviewTask : msg
       , onToggleViewMode : msg
       , onDragStart : Int -> msg
       , onDragEnd : msg
       , onDragOver : msg
       , onDropOnStatus : String -> msg
       , onBoardDropCommentChange : String -> msg
       , onSubmitBoardDrop : msg
       , onCancelBoardDrop : msg
       , onVoiceStartRecording : VoiceMode -> msg
       , onVoiceStopRecording : msg
       , onVoiceReset : msg
       , onVoiceApplyToForm : msg
       }
    -> Html msg
view project tasks documents activity analytics tab showTaskForm taskForm showDocForm docForm filters viewMode draggingTaskId boardDropTarget boardDropComment isBusy voiceState config =
    case project of
        NotAsked ->
            UI.emptyState "Loading..."

        Loading ->
            UI.loadingSpinner

        Failure err ->
            UI.emptyState ("Error: " ++ err)

        Success proj ->
            div []
                [ -- Back navigation + project header
                  div
                    [ style "margin-bottom" "1.5rem"
                    , style "padding-top" "clamp(1.25rem, 4vw, 2rem)"
                    ]
                    [ UI.backButton (config.onNavigate ProjectsRoute)
                    ]
                , projectHeader proj config.onRefresh
                , -- Analytics stats
                  viewAnalytics analytics
                , -- Tabs
                  Components.Tab.tabBar
                    [ Components.Tab.tab "Tasks" (tab == TasksTab) (config.onTabChange TasksTab)
                    , Components.Tab.tab "Documents" (tab == DocumentsTab) (config.onTabChange DocumentsTab)
                    , Components.Tab.tab "Activity" (tab == ActivityTab) (config.onTabChange ActivityTab)
                    ]
                , -- Tab content
                  case tab of
                    TasksTab ->
                        viewTasksTab tasks showTaskForm taskForm filters viewMode draggingTaskId boardDropTarget boardDropComment isBusy voiceState config

                    DocumentsTab ->
                        viewDocumentsTab documents showDocForm docForm isBusy config

                    ActivityTab ->
                        viewActivityTab activity
                ]


projectHeader : WorkProject -> msg -> Html msg
projectHeader project onRefresh =
    div
        [ style "margin-bottom" "1.5rem"
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "flex-start"
            , style "flex-wrap" "wrap"
            , style "gap" "1rem"
            , style "margin-bottom" "1rem"
            ]
            [ div []
                [ div
                    [ style "display" "flex"
                    , style "align-items" "baseline"
                    , style "gap" "0.75rem"
                    ]
                    [ h2
                        [ style "font-family" UI.fontDisplay
                        , style "font-size" "clamp(1.25rem, 5vw, 1.75rem)"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.02em"
                        , style "color" UI.colors.textPrimary
                        , style "margin" "0"
                        ]
                        [ text project.name ]
                    , span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.6875rem"
                        , style "color" UI.colors.textMuted
                        ]
                        [ text ("#" ++ String.fromInt project.id) ]
                    ]
                , if not (String.isEmpty project.description) then
                    div
                        [ style "color" UI.colors.textSecondary
                        , style "font-size" "0.875rem"
                        , style "margin-top" "0.5rem"
                        , style "max-width" "600px"
                        ]
                        [ Components.Markdown.view project.description ]
                  else
                    text ""
                ]
            , UI.button_ [ onClick onRefresh, title "Refresh" ] "REFRESH"
            ]
        , -- Tags + status
          div
            [ style "display" "flex"
            , style "flex-wrap" "wrap"
            , style "gap" "0.5rem"
            , style "align-items" "center"
            ]
            ([ if project.isActive then
                UI.pillBadge UI.colors.successDim UI.colors.success "ACTIVE"
               else
                UI.pillBadge UI.colors.borderLight UI.colors.textMuted "INACTIVE"
             ]
                ++ List.map UI.tagChip project.tags
            )
        ]


viewAnalytics : RemoteData TaskAnalytics -> Html msg
viewAnalytics analyticsData =
    case analyticsData of
        Success analytics ->
            div
                [ style "display" "grid"
                , style "grid-template-columns" "repeat(auto-fit, minmax(120px, 1fr))"
                , style "gap" "0.75rem"
                , style "margin-bottom" "1.5rem"
                ]
                (List.map
                    (\sc ->
                        UI.miniStat
                            (taskStatusLabel sc.status)
                            sc.count
                            (UI.taskStatusColor sc.status)
                    )
                    analytics.statusCounts
                )

        _ ->
            text ""


-- Tasks tab

viewTasksTab :
    RemoteData (List WorkTask)
    -> Bool
    -> TaskForm
    -> TaskFilters
    -> TaskViewMode
    -> Maybe Int
    -> Maybe ( Int, String )
    -> String
    -> Bool
    -> VoiceState
    -> { a
        | onNavigate : Route -> msg
        , onToggleTaskForm : msg
        , onTakeNextTask : msg
        , onTakeNextReviewTask : msg
        , onTaskTitleChange : String -> msg
        , onTaskDescChange : String -> msg
        , onTaskPriorityChange : String -> msg
        , onSubmitTask : msg
        , onCloseTaskForm : msg
        , onToggleStatus : String -> msg
        , onClearFilters : msg
        , onToggleViewMode : msg
        , onDragStart : Int -> msg
        , onDragEnd : msg
        , onDragOver : msg
        , onDropOnStatus : String -> msg
        , onBoardDropCommentChange : String -> msg
        , onSubmitBoardDrop : msg
        , onCancelBoardDrop : msg
        , onVoiceStartRecording : VoiceMode -> msg
        , onVoiceStopRecording : msg
        , onVoiceReset : msg
        , onVoiceApplyToForm : msg
       }
    -> Html msg
viewTasksTab tasks showForm taskForm filters viewMode draggingTaskId boardDropTarget boardDropComment isBusy voiceState config =
    div []
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-bottom" "1rem"
            ]
            [ UI.sectionHeader "Tasks"
            , div [ style "display" "flex", style "gap" "0.5rem" ]
                [ UI.button_
                    [ onClick config.onToggleViewMode
                    , style "padding" "0.375rem 0.75rem"
                    , style "font-size" "0.625rem"
                    ]
                    (case viewMode of
                        ListView -> "Board View"
                        BoardView -> "List View"
                    )
                , UI.button_ [ onClick config.onTakeNextReviewTask, disabled isBusy ] "Take Review"
                , UI.button_ [ onClick config.onTakeNextTask, disabled isBusy ] "Take Next"
                , UI.primaryButton [ onClick config.onToggleTaskForm, disabled isBusy ] "New Task"
                ]
            ]
        , if showForm then
            taskFormView taskForm isBusy voiceState config
          else
            text ""
        , case viewMode of
            ListView ->
                Components.TaskFilters.view filters
                    config.onToggleStatus
                    config.onClearFilters

            BoardView ->
                text ""
        , -- Board drop comment modal
          case boardDropTarget of
            Just ( _, _ ) ->
                boardDropCommentModal boardDropComment isBusy config

            Nothing ->
                text ""
        , case tasks of
            NotAsked ->
                UI.emptyState "Loading..."

            Loading ->
                UI.loadingSpinner

            Failure err ->
                UI.emptyState ("Error: " ++ err)

            Success taskList ->
                let
                    filtered = applyFilters filters taskList
                in
                if List.isEmpty filtered then
                    UI.emptyState "No tasks match the current filters."
                else
                    case viewMode of
                        ListView ->
                            div
                                [ style "display" "flex"
                                , style "flex-direction" "column"
                                , style "gap" "0.5rem"
                                ]
                                (List.map
                                    (\t -> Components.TaskCard.taskCard t (config.onNavigate (TaskDetailRoute t.id)))
                                    filtered
                                )

                        BoardView ->
                            viewBoardView filtered draggingTaskId config
        ]


viewBoardView :
    List WorkTask
    -> Maybe Int
    -> { a | onNavigate : Route -> msg, onDragStart : Int -> msg, onDragEnd : msg, onDragOver : msg, onDropOnStatus : String -> msg }
    -> Html msg
viewBoardView tasks draggingTaskId config =
    let
        statuses =
            [ "todo", "in_progress", "ready_for_review", "under_review", "done", "blocked" ]

        tasksByStatus status =
            List.filter (\t -> t.status == status) tasks
    in
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fill, minmax(200px, 1fr))"
        , style "gap" "0.75rem"
        , style "min-height" "200px"
        ]
        (List.map (\s -> statusColumn s (tasksByStatus s) draggingTaskId config) statuses)


statusColumn :
    String
    -> List WorkTask
    -> Maybe Int
    -> { a | onNavigate : Route -> msg, onDragStart : Int -> msg, onDragEnd : msg, onDragOver : msg, onDropOnStatus : String -> msg }
    -> Html msg
statusColumn status tasks draggingTaskId config =
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
            (List.map (\t -> boardTaskCard t config) tasks)
        ]


boardTaskCard :
    WorkTask
    -> { a | onNavigate : Route -> msg, onDragStart : Int -> msg, onDragEnd : msg }
    -> Html msg
boardTaskCard task config =
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
        , div [ style "display" "flex", style "gap" "0.25rem" ]
            [ Components.TaskCard.taskPriorityBadge task.priority ]
        ]


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


applyFilters : TaskFilters -> List WorkTask -> List WorkTask
applyFilters filters tasks =
    if List.isEmpty filters.statusFilter then
        tasks
    else
        List.filter (\t -> List.member t.status filters.statusFilter) tasks


taskFormView :
    TaskForm
    -> Bool
    -> VoiceState
    -> { a
        | onTaskTitleChange : String -> msg
        , onTaskDescChange : String -> msg
        , onTaskPriorityChange : String -> msg
        , onSubmitTask : msg
        , onCloseTaskForm : msg
        , onVoiceStartRecording : VoiceMode -> msg
        , onVoiceStopRecording : msg
        , onVoiceReset : msg
        , onVoiceApplyToForm : msg
       }
    -> Html msg
taskFormView form isBusy voiceState config =
    UI.card [ style "margin-bottom" "1rem" ]
        [ UI.cardHeader "Create Task"
        , div
            [ style "display" "flex", style "flex-direction" "column", style "gap" "1rem" ]
            [ -- Voice input section
              div
                [ style "background" "#0d1117"
                , style "border" "1px solid #1a2332"
                , style "border-radius" "6px"
                , style "padding" "0.75rem"
                ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    , style "margin-bottom" "0.5rem"
                    ]
                    [ span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.6875rem"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.08em"
                        , style "text-transform" "uppercase"
                        , style "color" UI.colors.textMuted
                        ]
                        [ text "Voice Input" ]
                    ]
                , Components.Voice.view voiceState
                    { onStartRecording = config.onVoiceStartRecording
                    , onStopRecording = config.onVoiceStopRecording
                    , onReset = config.onVoiceReset
                    }
                , case voiceState.recordingState of
                    VoiceDone _ ->
                        div [ style "margin-top" "0.5rem" ]
                            [ UI.primaryButton
                                [ onClick config.onVoiceApplyToForm
                                , style "font-size" "0.8rem"
                                , style "padding" "0.375rem 0.75rem"
                                ]
                                "Apply to Form"
                            ]

                    _ ->
                        text ""
                ]
            , -- Manual form fields
              UI.formField "Title" (UI.inputField form.title config.onTaskTitleChange "Task title")
            , UI.formField "Description" (UI.textareaField form.description config.onTaskDescChange "Task description")
            , UI.formField "Priority" (UI.selectField form.priority config.onTaskPriorityChange
                [ ( "low", "Low" ), ( "medium", "Medium" ), ( "high", "High" ), ( "critical", "Critical" ) ])
            , div
                [ style "display" "flex", style "gap" "0.75rem", style "justify-content" "flex-end" ]
                [ UI.button_ [ onClick config.onCloseTaskForm, disabled isBusy ] "Cancel"
                , UI.primaryButton
                    [ onClick config.onSubmitTask
                    , disabled (isBusy || String.isEmpty (String.trim form.title))
                    ]
                    (if isBusy then "Creating..." else "Create")
                ]
            ]
        ]


-- Documents tab

viewDocumentsTab :
    RemoteData (List WorkDocument)
    -> Bool
    -> DocumentForm
    -> Bool
    -> { a
        | onNavigate : Route -> msg
        , onToggleDocForm : msg
        , onDocTitleChange : String -> msg
        , onDocContentChange : String -> msg
        , onDocTypeChange : String -> msg
        , onSubmitDoc : msg
        , onCloseDocForm : msg
       }
    -> Html msg
viewDocumentsTab documents showForm docForm isBusy config =
    div []
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-bottom" "1rem"
            ]
            [ UI.sectionHeader "Documents"
            , UI.primaryButton [ onClick config.onToggleDocForm, disabled isBusy ] "New Document"
            ]
        , if showForm then
            docFormView docForm isBusy config
          else
            text ""
        , case documents of
            NotAsked ->
                UI.emptyState "Loading..."

            Loading ->
                UI.loadingSpinner

            Failure err ->
                UI.emptyState ("Error: " ++ err)

            Success docList ->
                if List.isEmpty docList then
                    UI.emptyState "No documents yet."
                else
                    div
                        [ style "display" "flex"
                        , style "flex-direction" "column"
                        , style "gap" "0.5rem"
                        ]
                        (List.map (\d -> documentCard d (config.onNavigate (DocumentDetailRoute d.id))) docList)
        ]


documentCard : WorkDocument -> msg -> Html msg
documentCard doc onSelect =
    div
        [ onClick onSelect
        , style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1rem 1.25rem"
        , style "cursor" "pointer"
        , style "transition" "all 0.15s ease"
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            ]
            [ div [ style "display" "flex", style "gap" "0.75rem", style "align-items" "center" ]
                [ span
                    [ style "font-size" "0.9375rem"
                    , style "font-weight" "500"
                    , style "color" UI.colors.textPrimary
                    ]
                    [ text doc.title ]
                , UI.docTypeBadge doc.documentType
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    , style "color" UI.colors.textMuted
                    ]
                    [ text ("v" ++ String.fromInt doc.version) ]
                ]
            , UI.timestamp doc.updatedAt
            ]
        ]


docFormView :
    DocumentForm
    -> Bool
    -> { a
        | onDocTitleChange : String -> msg
        , onDocContentChange : String -> msg
        , onDocTypeChange : String -> msg
        , onSubmitDoc : msg
        , onCloseDocForm : msg
       }
    -> Html msg
docFormView form isBusy config =
    UI.card [ style "margin-bottom" "1rem" ]
        [ UI.cardHeader "Create Document"
        , div
            [ style "display" "flex", style "flex-direction" "column", style "gap" "1rem" ]
            [ div
                [ style "display" "grid"
                , style "grid-template-columns" "2fr 1fr"
                , style "gap" "1rem"
                ]
                [ UI.formField "Title" (UI.inputField form.title config.onDocTitleChange "Document title")
                , UI.formField "Type" (UI.selectField form.documentType config.onDocTypeChange
                    [ ( "notes", "Notes" ), ( "plan", "Plan" ), ( "specification", "Specification" ), ( "code", "Code" ), ( "other", "Other" ) ])
                ]
            , UI.formField "Content" (UI.textareaField form.content config.onDocContentChange "Document content (markdown)")
            , div
                [ style "display" "flex", style "gap" "0.75rem", style "justify-content" "flex-end" ]
                [ UI.button_ [ onClick config.onCloseDocForm, disabled isBusy ] "Cancel"
                , UI.primaryButton
                    [ onClick config.onSubmitDoc
                    , disabled (isBusy || String.isEmpty (String.trim form.title))
                    ]
                    (if isBusy then "Creating..." else "Create")
                ]
            ]
        ]


-- Activity tab

viewActivityTab : RemoteData (List ActivityLog) -> Html msg
viewActivityTab activity =
    case activity of
        NotAsked ->
            UI.emptyState "Loading..."

        Loading ->
            UI.loadingSpinner

        Failure err ->
            UI.emptyState ("Error: " ++ err)

        Success logs ->
            if List.isEmpty logs then
                UI.emptyState "No recent activity."
            else
                div
                    [ style "display" "flex"
                    , style "flex-direction" "column"
                    , style "gap" "0.375rem"
                    ]
                    (List.map activityItem logs)


activityItem : ActivityLog -> Html msg
activityItem log =
    div
        [ style "display" "flex"
        , style "gap" "0.75rem"
        , style "align-items" "flex-start"
        , style "padding" "0.5rem 0.75rem"
        , style "background-color" UI.colors.bgSurface
        , style "border-radius" "3px"
        , style "font-size" "0.8125rem"
        ]
        [ -- Action badge
          span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.5625rem"
            , style "font-weight" "600"
            , style "color" (actionColor log.action)
            , style "padding" "0.125rem 0.375rem"
            , style "background-color" (actionBgColor log.action)
            , style "border-radius" "2px"
            , style "letter-spacing" "0.05em"
            , style "white-space" "nowrap"
            , style "margin-top" "0.125rem"
            ]
            [ text (String.toUpper (String.replace "_" " " log.action)) ]
        , -- Details
          div [ style "flex" "1", style "min-width" "0" ]
            [ span [ style "color" UI.colors.textSecondary ] [ text log.details ]
            ]
        , -- Timestamp
          span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.625rem"
            , style "color" UI.colors.textMuted
            , style "white-space" "nowrap"
            ]
            [ text (UI.formatTime log.createdAt) ]
        ]


actionColor : String -> String
actionColor action =
    case action of
        "created" -> UI.colors.success
        "updated" -> UI.colors.accent
        "deleted" -> UI.colors.error
        "status_changed" -> UI.colors.warning
        "commented" -> "#60a5fa"
        _ -> UI.colors.textSecondary


actionBgColor : String -> String
actionBgColor action =
    case action of
        "created" -> UI.colors.successDim
        "updated" -> UI.colors.accentDim
        "deleted" -> UI.colors.errorDim
        "status_changed" -> UI.colors.warningDim
        "commented" -> "rgba(96, 165, 250, 0.12)"
        _ -> UI.colors.borderLight
