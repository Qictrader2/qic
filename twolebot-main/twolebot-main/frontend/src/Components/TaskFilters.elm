module Components.TaskFilters exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


view :
    TaskFilters
    -> (String -> msg)
    -> msg
    -> Html msg
view filters onToggleStatus onClear =
    div
        [ style "display" "flex"
        , style "flex-wrap" "wrap"
        , style "gap" "0.75rem"
        , style "align-items" "center"
        , style "padding" "0.75rem 1rem"
        , style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "margin-bottom" "1rem"
        ]
        [ filterGroup "Status"
            [ filterChip "todo" "Todo" (List.member "todo" filters.statusFilter) (onToggleStatus "todo")
            , filterChip "in_progress" "Active" (List.member "in_progress" filters.statusFilter) (onToggleStatus "in_progress")
            , filterChip "ready_for_review" "Review" (List.member "ready_for_review" filters.statusFilter) (onToggleStatus "ready_for_review")
            , filterChip "under_review" "Under Review" (List.member "under_review" filters.statusFilter) (onToggleStatus "under_review")
            , filterChip "done" "Done" (List.member "done" filters.statusFilter) (onToggleStatus "done")
            , filterChip "blocked" "Blocked" (List.member "blocked" filters.statusFilter) (onToggleStatus "blocked")
            , filterChip "abandoned" "Abandoned" (List.member "abandoned" filters.statusFilter) (onToggleStatus "abandoned")
            , filterChip "archived" "Archived" (List.member "archived" filters.statusFilter) (onToggleStatus "archived")
            ]
        , if not (List.isEmpty filters.statusFilter) then
            button
                [ onClick onClear
                , style "background" "transparent"
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "color" UI.colors.textMuted
                , style "padding" "0.25rem 0.625rem"
                , style "border-radius" "2px"
                , style "cursor" "pointer"
                , style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "letter-spacing" "0.05em"
                ]
                [ text "CLEAR" ]
          else
            text ""
        ]


filterGroup : String -> List (Html msg) -> Html msg
filterGroup label chips =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.375rem"
        ]
        (span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.5625rem"
            , style "color" UI.colors.textMuted
            , style "letter-spacing" "0.1em"
            , style "text-transform" "uppercase"
            , style "margin-right" "0.25rem"
            ]
            [ text label ]
            :: chips
        )


filterChip : String -> String -> Bool -> msg -> Html msg
filterChip _ label isActive onToggle =
    button
        [ onClick onToggle
        , style "background-color" (if isActive then UI.colors.accentDim else "transparent")
        , style "color" (if isActive then UI.colors.accent else UI.colors.textMuted)
        , style "border" ("1px solid " ++ (if isActive then "rgba(0, 212, 170, 0.3)" else UI.colors.border))
        , style "padding" "0.1875rem 0.5rem"
        , style "border-radius" "2px"
        , style "cursor" "pointer"
        , style "font-family" UI.fontMono
        , style "font-size" "0.625rem"
        , style "font-weight" (if isActive then "600" else "400")
        , style "letter-spacing" "0.05em"
        , style "transition" "none"
        , style "transform" "none"
        , style "filter" "none"
        ]
        [ text label ]
