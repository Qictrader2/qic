module Components.Tab exposing (tab, tabBar)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import UI


tabBar : List (Html msg) -> Html msg
tabBar tabs =
    div
        [ style "display" "flex"
        , style "gap" "0.25rem"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        , style "margin-bottom" "1.5rem"
        ]
        tabs


tab : String -> Bool -> msg -> Html msg
tab label isActive onSelect =
    button
        [ onClick onSelect
        , style "background" "transparent"
        , style "color" (if isActive then UI.colors.accent else UI.colors.textSecondary)
        , style "border" "none"
        , style "border-bottom"
            (if isActive then
                "2px solid " ++ UI.colors.accent
             else
                "2px solid transparent"
            )
        , style "padding" "0.75rem 1.25rem"
        , style "cursor" "pointer"
        , style "font-family" UI.fontMono
        , style "font-size" "0.75rem"
        , style "font-weight" (if isActive then "600" else "500")
        , style "letter-spacing" "0.08em"
        , style "text-transform" "uppercase"
        , style "transition" "none"
        , style "transform" "none"
        , style "filter" "none"
        ]
        [ text label ]
