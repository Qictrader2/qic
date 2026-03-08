module Components.Pagination exposing (Config, view, viewCompact)

import Html exposing (Html, button, div, span, text)
import Html.Attributes exposing (attribute, disabled, style, title)
import Html.Events exposing (onClick)
import UI


type alias Config msg =
    { page : Int
    , totalPages : Int
    , onPageChange : Int -> msg
    }


type Size
    = Full
    | Compact


type PageItem
    = Page Int
    | Gap


view : Config msg -> Html msg
view config =
    viewInternal Full config


viewCompact : Config msg -> Html msg
viewCompact config =
    viewInternal Compact config


viewInternal : Size -> Config msg -> Html msg
viewInternal size config =
    if config.totalPages <= 1 then
        text ""
    else
        let
            isCompact =
                size == Compact

            radius =
                if isCompact then
                    1
                else
                    2

            items =
                pageItems config.totalPages config.page radius

            containerStyles =
                [ style "display" "flex"
                , style "justify-content" (if isCompact then "flex-end" else "center")
                , style "align-items" "center"
                , style "gap" (if isCompact then "0.25rem" else "0.375rem")
                , style "margin-top" (if isCompact then "0" else "2rem")
                , style "padding" (if isCompact then "0" else "1rem")
                ]

            railStyles =
                [ style "display" "inline-flex"
                , style "align-items" "center"
                , style "gap" (if isCompact then "0.25rem" else "0.25rem")
                , style "padding" (if isCompact then "0.25rem" else "0.25rem")
                , style "max-width" "100%"
                , style "overflow" "visible"
                , style "background"
                    ("linear-gradient(180deg, "
                        ++ UI.colors.bgTertiary
                        ++ " 0%, "
                        ++ UI.colors.bgSecondary
                        ++ " 100%)"
                    )
                , style "border" ("1px solid " ++ UI.colors.border)
                , style "border-radius" "8px"
                , style "box-shadow"
                    ("0 0 0 1px rgba(0, 212, 170, 0.06), 0 10px 30px rgba(0,0,0,0.35)")
                ]

            prefix =
                if isCompact then
                    [ navButton True "FIRST" "«" 0 (config.page == 0) config
                    , navButton True "PREV" "‹" (config.page - 1) (config.page == 0) config
                    ]
                else
                    [ navButton False "FIRST" "«" 0 (config.page == 0) config
                    , navButton False "PREV" "‹" (config.page - 1) (config.page == 0) config
                    ]

            middle =
                if isCompact then
                    []
                else
                    List.map (viewItem False config) items

            suffix =
                if isCompact then
                    [ navButton True "NEXT" "›" (config.page + 1) (config.page >= config.totalPages - 1) config
                    , navButton True "LAST" "»" (config.totalPages - 1) (config.page >= config.totalPages - 1) config
                    ]
                else
                    [ navButton False "NEXT" "›" (config.page + 1) (config.page >= config.totalPages - 1) config
                    , navButton False "LAST" "»" (config.totalPages - 1) (config.page >= config.totalPages - 1) config
                    ]
        in
        div containerStyles
            [ div railStyles
                (List.concat [ prefix, middle, suffix ])
            ]


navButton : Bool -> String -> String -> Int -> Bool -> Config msg -> Html msg
navButton isCompact label glyph targetPage isDisabled config =
    let
        padding =
            if isCompact then
                "0.25rem 0.5rem"
            else
                "0.35rem 0.55rem"

        fontSize =
            if isCompact then
                "0.75rem"
            else
                "0.8125rem"

        borderColor =
            if isDisabled then
                UI.colors.border
            else
                "rgba(0, 212, 170, 0.18)"

        bg =
            if isDisabled then
                UI.colors.bgSurface
            else
                UI.colors.bgTertiary
    in
    button
        ([ title label
         , style "padding" padding
         , style "background-color" bg
         , style "border" ("1px solid " ++ borderColor)
         , style "border-radius" "6px"
         , style "color" (if isDisabled then UI.colors.textMuted else UI.colors.textSecondary)
         , style "font-family" UI.fontMono
         , style "font-weight" "700"
         , style "font-size" fontSize
         , style "line-height" "1"
         , style "letter-spacing" "0.04em"
         , style "min-width" (if isCompact then "2rem" else "1.75rem")
         , style "cursor" (if isDisabled then "not-allowed" else "pointer")
         , style "opacity" (if isDisabled then "0.55" else "1")
         , attribute "aria-label" label
         ]
            ++ (if isDisabled then
                    [ disabled True ]
                else
                    [ onClick (config.onPageChange (clampPage config.totalPages targetPage)) ]
               )
        )
        [ text glyph ]


viewItem : Bool -> Config msg -> PageItem -> Html msg
viewItem isCompact config item =
    case item of
        Gap ->
            span
                [ style "padding" (if isCompact then "0 0.25rem" else "0 0.375rem")
                , style "color" UI.colors.textMuted
                , style "font-family" UI.fontMono
                , style "font-size" (if isCompact then "0.6875rem" else "0.75rem")
                , style "letter-spacing" "0.06em"
                ]
                [ text "…" ]

        Page p ->
            pageButton isCompact (p == config.page) p config


pageButton : Bool -> Bool -> Int -> Config msg -> Html msg
pageButton isCompact isActive pageIndex config =
    let
        padding =
            if isCompact then
                "0.25rem 0.5rem"
            else
                "0.35rem 0.55rem"

        fontSize =
            if isCompact then
                "0.6875rem"
            else
                "0.75rem"

        bg =
            if isActive then
                ("linear-gradient(135deg, "
                    ++ UI.colors.accent
                    ++ " 0%, #00a884 100%)"
                )
            else
                UI.colors.bgSurface

        textColor =
            if isActive then
                UI.colors.bgPrimary
            else
                UI.colors.textSecondary

        borderColor =
            if isActive then
                "rgba(0, 212, 170, 0.8)"
            else
                UI.colors.border

        shadow =
            if isActive then
                ("0 0 0 1px rgba(0, 212, 170, 0.25), 0 0 18px "
                    ++ UI.colors.accentGlow
                )
            else
                "none"
    in
    button
        [ title ("Page " ++ String.fromInt (pageIndex + 1))
        , onClick (config.onPageChange pageIndex)
        , style "padding" padding
        , style "background" bg
        , style "border" ("1px solid " ++ borderColor)
        , style "border-radius" "6px"
        , style "color" textColor
        , style "font-family" UI.fontMono
        , style "font-weight" (if isActive then "800" else "600")
        , style "font-size" fontSize
        , style "letter-spacing" "0.04em"
        , style "min-width" (if isCompact then "2rem" else "1.75rem")
        , style "text-align" "center"
        , style "box-shadow" shadow
        , style "cursor" "pointer"
        , attribute "aria-current" (if isActive then "page" else "false")
        ]
        [ text (String.fromInt (pageIndex + 1)) ]


pageItems : Int -> Int -> Int -> List PageItem
pageItems totalPages currentPage radius =
    let
        total =
            Basics.max 1 totalPages

        current =
            clampPage total currentPage

        maxWithoutGaps =
            (2 * radius) + 5

        start =
            Basics.max 1 (current - radius)

        finish =
            Basics.min (total - 2) (current + radius)

        leftExtras =
            if start <= 2 then
                List.map Page (List.range 1 (start - 1))
            else
                [ Gap ]

        rightExtras =
            if finish >= total - 3 then
                List.map Page (List.range (finish + 1) (total - 2))
            else
                [ Gap ]

        middle =
            List.map Page (List.range start finish)
    in
    if total <= maxWithoutGaps then
        List.map Page (List.range 0 (total - 1))
    else
        Page 0 :: (leftExtras ++ middle ++ rightExtras ++ [ Page (total - 1) ])


clampPage : Int -> Int -> Int
clampPage totalPages page =
    let
        last =
            Basics.max 0 (totalPages - 1)
    in
    if page < 0 then
        0
    else if page > last then
        last
    else
        page

