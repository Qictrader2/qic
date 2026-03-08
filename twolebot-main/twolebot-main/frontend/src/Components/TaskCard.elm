module Components.TaskCard exposing (taskCard, taskStatusBadge, taskPriorityBadge)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


taskCard : WorkTask -> msg -> Html msg
taskCard task onSelect =
    div
        [ onClick onSelect
        , style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1rem 1.25rem"
        , style "cursor" "pointer"
        , style "transition" "all 0.15s ease"
        , style "border-left" ("3px solid " ++ UI.taskStatusColor task.status)
        ]
        [ -- Header: title + ID
          div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "flex-start"
            , style "gap" "0.75rem"
            , style "margin-bottom" "0.5rem"
            ]
            [ span
                [ style "font-size" "0.9375rem"
                , style "font-weight" "500"
                , style "color" UI.colors.textPrimary
                , style "line-height" "1.4"
                ]
                [ text task.title ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "color" UI.colors.textMuted
                , style "white-space" "nowrap"
                ]
                [ text ("#" ++ String.fromInt task.id) ]
            ]
        , -- Description preview
          if not (String.isEmpty task.description) then
            div
                [ style "color" UI.colors.textMuted
                , style "font-size" "0.8125rem"
                , style "line-height" "1.5"
                , style "margin-bottom" "0.75rem"
                , style "overflow" "hidden"
                , style "text-overflow" "ellipsis"
                , style "display" "-webkit-box"
                , style "-webkit-line-clamp" "2"
                , style "-webkit-box-orient" "vertical"
                ]
                [ text (UI.truncateText 200 task.description) ]
          else
            text ""
        , -- Badges row
          div
            [ style "display" "flex"
            , style "flex-wrap" "wrap"
            , style "gap" "0.5rem"
            , style "align-items" "center"
            ]
            [ taskStatusBadge task.status
            , taskPriorityBadge task.priority
            , if not (List.isEmpty task.tags) then
                div
                    [ style "display" "flex"
                    , style "gap" "0.25rem"
                    , style "flex-wrap" "wrap"
                    ]
                    (List.map tagBadge task.tags)
              else
                text ""
            ]
        ]


taskStatusBadge : String -> Html msg
taskStatusBadge status =
    let
        ( bgColor, textColor, label ) =
            case status of
                "todo" -> ( UI.colors.borderLight, UI.colors.textMuted, "TODO" )
                "in_progress" -> ( UI.colors.warningDim, UI.colors.warning, "IN PROGRESS" )
                "ready_for_review" -> ( "rgba(0, 212, 170, 0.12)", UI.colors.accent, "REVIEW READY" )
                "under_review" -> ( "rgba(139, 92, 246, 0.12)", "#a78bfa", "UNDER REVIEW" )
                "done" -> ( UI.colors.successDim, UI.colors.success, "DONE" )
                "blocked" -> ( UI.colors.errorDim, UI.colors.error, "BLOCKED" )
                "abandoned" -> ( UI.colors.borderLight, UI.colors.textMuted, "ABANDONED" )
                _ -> ( UI.colors.borderLight, UI.colors.textMuted, String.toUpper status )
    in
    UI.pillBadge bgColor textColor label


taskPriorityBadge : String -> Html msg
taskPriorityBadge priority =
    let
        ( bgColor, textColor, label ) =
            case priority of
                "low" -> ( "rgba(107, 138, 173, 0.12)", "#6b8aad", "LOW" )
                "medium" -> ( "rgba(160, 174, 192, 0.12)", "#a0aec0", "MEDIUM" )
                "high" -> ( "rgba(245, 158, 11, 0.12)", "#f59e0b", "HIGH" )
                "critical" -> ( "rgba(248, 113, 113, 0.12)", "#f87171", "CRITICAL" )
                _ -> ( UI.colors.borderLight, UI.colors.textMuted, String.toUpper priority )
    in
    UI.pillBadge bgColor textColor label


tagBadge : String -> Html msg
tagBadge tag =
    span
        [ style "font-family" UI.fontMono
        , style "font-size" "0.5625rem"
        , style "color" UI.colors.textSecondary
        , style "padding" "0.125rem 0.375rem"
        , style "background-color" UI.colors.borderLight
        , style "border-radius" "2px"
        , style "letter-spacing" "0.03em"
        ]
        [ text tag ]
