module Pages.Projects exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


view :
    RemoteData (List WorkProject)
    -> Bool
    -> ProjectForm
    -> Bool
    -> (Route -> msg)
    -> msg
    -> (String -> msg)
    -> (String -> msg)
    -> (String -> msg)
    -> (String -> msg)
    -> msg
    -> msg
    -> msg
    -> Html msg
view projects showForm form isBusy onNavigate onRefresh onNameChange onDescChange onTagsChange onGitRemoteChange onToggleForm onSubmit onCloseForm =
    div []
        [ UI.pageHeader "Projects"
            [ UI.primaryButton [ onClick onToggleForm, disabled isBusy ] "New Project"
            , UI.button_ [ onClick onRefresh, title "Refresh Projects", disabled isBusy ] "Refresh"
            ]
        , if showForm then
            projectFormView form isBusy onNameChange onDescChange onTagsChange onGitRemoteChange onSubmit onCloseForm
          else
            text ""
        , case projects of
            NotAsked ->
                UI.emptyState "Loading projects..."

            Loading ->
                UI.loadingSpinner

            Failure err ->
                UI.emptyState ("Error: " ++ err)

            Success projectList ->
                if List.isEmpty projectList then
                    UI.emptyState "No projects yet. Create one to get started."
                else
                    div
                        [ style "display" "grid"
                        , style "grid-template-columns" "repeat(auto-fill, minmax(340px, 1fr))"
                        , style "gap" "1rem"
                        ]
                        (List.map (projectCard onNavigate) projectList)
        ]


projectCard : (Route -> msg) -> WorkProject -> Html msg
projectCard onNavigate project =
    div
        [ onClick (onNavigate (ProjectDetailRoute project.id))
        , style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        , style "cursor" "pointer"
        , style "transition" "all 0.15s ease"
        , style "position" "relative"
        , style "overflow" "hidden"
        ]
        [ -- Top accent line
          div
            [ style "position" "absolute"
            , style "top" "0"
            , style "left" "0"
            , style "right" "0"
            , style "height" "1px"
            , style "background" ("linear-gradient(90deg, " ++ UI.colors.accent ++ ", transparent 60%)")
            , style "opacity" "0.5"
            ]
            []
        , -- Header
          div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "flex-start"
            , style "margin-bottom" "0.75rem"
            ]
            [ h3
                [ style "font-size" "1rem"
                , style "font-weight" "600"
                , style "color" UI.colors.textPrimary
                , style "margin" "0"
                ]
                [ text project.name ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "color" UI.colors.textMuted
                ]
                [ text ("#" ++ String.fromInt project.id) ]
            ]
        , -- Description
          if not (String.isEmpty project.description) then
            div
                [ style "color" UI.colors.textSecondary
                , style "font-size" "0.8125rem"
                , style "line-height" "1.5"
                , style "margin-bottom" "0.75rem"
                , style "overflow" "hidden"
                , style "text-overflow" "ellipsis"
                , style "display" "-webkit-box"
                , style "-webkit-line-clamp" "3"
                , style "-webkit-box-orient" "vertical"
                ]
                [ text (UI.truncateText 200 project.description) ]
          else
            text ""
        , -- Tags
          if not (List.isEmpty project.tags) then
            div
                [ style "display" "flex"
                , style "flex-wrap" "wrap"
                , style "gap" "0.375rem"
                , style "margin-bottom" "0.75rem"
                ]
                (List.map UI.tagChip project.tags)
          else
            text ""
        , -- Footer: status + task count + date
          div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            ]
            [ div [ style "display" "flex", style "gap" "0.5rem", style "align-items" "center" ]
                [ if project.isActive then
                    UI.pillBadge UI.colors.successDim UI.colors.success "ACTIVE"
                  else
                    UI.pillBadge UI.colors.borderLight UI.colors.textMuted "INACTIVE"
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "color" UI.colors.textSecondary
                    ]
                    [ text (String.fromInt project.taskCount ++ " tasks") ]
                ]
            , UI.timestamp project.updatedAt
            ]
        ]


projectFormView : ProjectForm -> Bool -> (String -> msg) -> (String -> msg) -> (String -> msg) -> (String -> msg) -> msg -> msg -> Html msg
projectFormView form isBusy onNameChange onDescChange onTagsChange onGitRemoteChange onSubmit onCancel =
    UI.card []
        [ UI.cardHeader "Create Project"
        , div
            [ style "display" "flex"
            , style "flex-direction" "column"
            , style "gap" "1rem"
            ]
            [ UI.formField "Name" (UI.inputField form.name onNameChange "Project name")
            , UI.formField "Description" (UI.textareaField form.description onDescChange "Project description")
            , UI.formField "Tags" (UI.inputField form.tags onTagsChange "Comma-separated tags")
            , UI.formField "Git Remote URL" (UI.inputField form.gitRemoteUrl onGitRemoteChange "https://github.com/org/repo.git")
            , div
                [ style "display" "flex"
                , style "gap" "0.75rem"
                , style "justify-content" "flex-end"
                ]
                [ UI.button_ [ onClick onCancel, disabled isBusy ] "Cancel"
                , UI.primaryButton
                    [ onClick onSubmit
                    , disabled (isBusy || String.isEmpty (String.trim form.name))
                    ]
                    (if isBusy then "Creating..." else "Create")
                ]
            ]
        ]
