module UI exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)


{-| TWOLEBOT UI SYSTEM
    Aesthetic: Industrial Command Console

    A bold, utilitarian dashboard for bot operators.
    Sharp geometry, high contrast accents, purposeful typography.
-}


-- ═══════════════════════════════════════════════════════════════════════════
-- COLOR SYSTEM
-- ═══════════════════════════════════════════════════════════════════════════


colors :
    { bgPrimary : String
    , bgSecondary : String
    , bgTertiary : String
    , bgSurface : String
    , textPrimary : String
    , textSecondary : String
    , textMuted : String
    , accent : String
    , accentGlow : String
    , accentDim : String
    , success : String
    , successDim : String
    , warning : String
    , warningDim : String
    , error : String
    , errorDim : String
    , border : String
    , borderLight : String
    , gridLine : String
    }
colors =
    { bgPrimary = "#0a0e14"      -- Deep void black
    , bgSecondary = "#11151c"    -- Slightly lifted
    , bgTertiary = "#1a1f2b"     -- Card surfaces
    , bgSurface = "#0d1117"      -- Inset panels
    , textPrimary = "#e6edf3"    -- High contrast white
    , textSecondary = "#9ca3af"  -- Readable secondary
    , textMuted = "#5c6370"      -- De-emphasized
    , accent = "#00d4aa"         -- Cyan-green terminal glow
    , accentGlow = "rgba(0, 212, 170, 0.25)"
    , accentDim = "rgba(0, 212, 170, 0.08)"
    , success = "#4ade80"        -- Bright operational green
    , successDim = "rgba(74, 222, 128, 0.12)"
    , warning = "#fbbf24"        -- Amber alert
    , warningDim = "rgba(251, 191, 36, 0.12)"
    , error = "#f87171"          -- Red alert
    , errorDim = "rgba(248, 113, 113, 0.12)"
    , border = "#21262d"         -- Subtle panel edges
    , borderLight = "rgba(255, 255, 255, 0.06)"
    , gridLine = "rgba(0, 212, 170, 0.06)"
    }


-- ═══════════════════════════════════════════════════════════════════════════
-- TYPOGRAPHY
-- ═══════════════════════════════════════════════════════════════════════════


{-| Font stack: IBM Plex for industrial precision, JetBrains Mono for data
-}
fontBody : String
fontBody =
    "'IBM Plex Sans', 'SF Pro Display', -apple-system, system-ui, sans-serif"


fontMono : String
fontMono =
    "'JetBrains Mono', 'IBM Plex Mono', 'SF Mono', Consolas, monospace"


fontDisplay : String
fontDisplay =
    "'IBM Plex Sans Condensed', 'Impact', 'Arial Black', sans-serif"


-- ═══════════════════════════════════════════════════════════════════════════
-- LAYOUT SHELL
-- ═══════════════════════════════════════════════════════════════════════════


appShell : List (Html msg) -> Html msg
appShell content =
    div
        [ style "min-height" "100vh"
        , style "background-color" colors.bgPrimary
        , style "background-image"
            ("linear-gradient(to bottom, " ++ colors.bgPrimary ++ ", #060810), " ++
             "repeating-linear-gradient(0deg, transparent, transparent 100px, " ++ colors.gridLine ++ " 100px, " ++ colors.gridLine ++ " 101px)")
        , style "color" colors.textPrimary
        , style "font-family" fontBody
        , style "line-height" "1.6"
        , style "font-size" "14px"
        , style "-webkit-font-smoothing" "antialiased"
        ]
        content


container : List (Html msg) -> Html msg
container content =
    div
        [ style "max-width" "1400px"
        , style "margin" "0 auto"
        , style "padding" "0 clamp(1rem, 4vw, 2rem)"
        ]
        content


-- ═══════════════════════════════════════════════════════════════════════════
-- HEADER / NAVIGATION
-- ═══════════════════════════════════════════════════════════════════════════


header : Bool -> Route -> (Route -> msg) -> Html msg
header backendOnline currentRoute onNavigate =
    Html.header
        [ style "background-color" colors.bgSecondary
        , style "border-bottom" ("1px solid " ++ colors.border)
        , style "padding" "0"
        , style "position" "sticky"
        , style "top" "0"
        , style "z-index" "100"
        , style "backdrop-filter" "blur(12px)"
        ]
        [ container
            [ div
                [ style "display" "flex"
                , style "justify-content" "space-between"
                , style "align-items" "stretch"
                , style "flex-wrap" "wrap"
                , style "gap" "0.75rem"
                , style "min-height" "64px"
                ]
                [ -- Logo / Brand
                  div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "clamp(0.75rem, 2vw, 1.25rem)"
                    , style "padding-right" "clamp(0.75rem, 3vw, 2rem)"
                    , style "border-right" ("1px solid " ++ colors.border)
                    , class "tb-brand-shell"
                    ]
                    [ div
                        [ style "display" "flex"
                        , style "align-items" "center"
                        , style "gap" "0.75rem"
                        ]
                        [ -- Geometric logo mark
                          div
                            [ style "width" "28px"
                            , style "height" "28px"
                            , style "background" ("linear-gradient(135deg, " ++ colors.accent ++ " 0%, #00a884 100%)")
                            , style "clip-path" "polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)"
                            ]
                            []
                        , h1
                            [ style "font-family" fontDisplay
                            , style "font-size" "clamp(1.0rem, 4vw, 1.25rem)"
                            , style "font-weight" "600"
                            , style "letter-spacing" "0.05em"
                            , style "text-transform" "uppercase"
                            , style "color" colors.textPrimary
                            , style "margin" "0"
                            ]
                            [ text "TWOLEBOT" ]
                        ]
                    , statusIndicator backendOnline
                    ]
                , -- Navigation (desktop)
                  nav
                    [ class "tb-desktop-nav"
                    , style "display" "flex"
                    , style "align-items" "stretch"
                    , style "flex-wrap" "wrap"
                    , style "justify-content" "flex-end"
                    , style "column-gap" "0.25rem"
                    , style "row-gap" "0.25rem"
                    , style "margin-left" "auto"
                    ]
                    [ navLink "Dashboard" DashboardRoute currentRoute onNavigate
                    , navLink "Chat" (ChatRoute Nothing) currentRoute onNavigate
                    , navLink "Messages" (MessagesRoute Nothing Nothing) currentRoute onNavigate
                    , navLink "Logs" LogsRoute currentRoute onNavigate
                    , navLink "Jobs" CronJobsRoute currentRoute onNavigate
                    , navDivider
                    , navLink "Projects" ProjectsRoute currentRoute onNavigate
                    , navLink "Live" LiveBoardRoute currentRoute onNavigate
                    , navDivider
                    , navLink "Settings" SettingsRoute currentRoute onNavigate
                    ]
                ]
            ]
        , mobileTabs currentRoute onNavigate
        ]


mobileTabs : Route -> (Route -> msg) -> Html msg
mobileTabs currentRoute onNavigate =
    let
        tab label route =
            mobileTab label route currentRoute onNavigate
    in
    div
        [ class "tb-mobile-tabs"
        , style "border-top" ("1px solid " ++ colors.border)
        , style "background-color" colors.bgSecondary
        ]
        [ container
            [ div
                [ style "position" "relative"
                , style "padding" "0.5rem 0"
                ]
                [ -- Scrollable tabs
                  div
                    [ class "tb-mobile-tabs__inner"
                    , style "display" "flex"
                    , style "gap" "0.5rem"
                    , style "overflow-x" "auto"
                    , style "overflow-y" "hidden"
                    , style "-webkit-overflow-scrolling" "touch"
                    , style "scrollbar-width" "none"
                    , style "padding" "0.25rem 0.75rem"
                    , style "margin" "0 -0.75rem"
                    ]
                    [ tab "Dashboard" DashboardRoute
                    , tab "Chat" (ChatRoute Nothing)
                    , tab "Messages" (MessagesRoute Nothing Nothing)
                    , tab "Logs" LogsRoute
                    , tab "Jobs" CronJobsRoute
                    , tab "Projects" ProjectsRoute
                    , tab "Live" LiveBoardRoute
                    , tab "Settings" SettingsRoute
                    ]

                -- Edge fades (purely visual)
                , div
                    [ style "pointer-events" "none"
                    , style "position" "absolute"
                    , style "left" "0"
                    , style "top" "0"
                    , style "bottom" "0"
                    , style "width" "24px"
                    , style "background" ("linear-gradient(90deg, " ++ colors.bgSecondary ++ ", transparent)")
                    ]
                    []
                , div
                    [ style "pointer-events" "none"
                    , style "position" "absolute"
                    , style "right" "0"
                    , style "top" "0"
                    , style "bottom" "0"
                    , style "width" "24px"
                    , style "background" ("linear-gradient(-90deg, " ++ colors.bgSecondary ++ ", transparent)")
                    ]
                    []
                ]
            ]
        ]


mobileTab : String -> Route -> Route -> (Route -> msg) -> Html msg
mobileTab label route currentRoute onNavigate =
    let
        isActive =
            case ( route, currentRoute ) of
                ( WelcomeRoute, WelcomeRoute ) -> True
                ( DashboardRoute, DashboardRoute ) -> True
                ( MessagesRoute _ _, MessagesRoute _ _ ) -> True
                ( LogsRoute, LogsRoute ) -> True
                ( CronJobsRoute, CronJobsRoute ) -> True
                ( SettingsRoute, SettingsRoute ) -> True
                ( CapabilitiesRoute, CapabilitiesRoute ) -> True
                ( ProjectsRoute, ProjectsRoute ) -> True
                ( ProjectsRoute, ProjectDetailRoute _ ) -> True
                ( LiveBoardRoute, LiveBoardRoute ) -> True
                ( ChatRoute _, ChatRoute _ ) -> True
                _ -> False

        bg =
            if isActive then
                colors.accentDim
            else
                colors.bgSurface

        borderColor =
            if isActive then
                "rgba(0, 212, 170, 0.45)"
            else
                colors.border

        textColor =
            if isActive then
                colors.accent
            else
                colors.textSecondary

        shadow =
            if isActive then
                "0 0 18px rgba(0, 212, 170, 0.18)"
            else
                "none"
    in
    button
        [ onClick (onNavigate route)
        , style "background-color" bg
        , style "color" textColor
        , style "border" ("1px solid " ++ borderColor)
        , style "border-radius" "999px"
        , style "padding" "0.5rem 0.875rem"
        , style "font-family" fontMono
        , style "font-size" "0.6875rem"
        , style "font-weight" "700"
        , style "letter-spacing" "0.08em"
        , style "text-transform" "uppercase"
        , style "white-space" "nowrap"
        , style "box-shadow" shadow
        ]
        [ text label ]


statusIndicator : Bool -> Html msg
statusIndicator online =
    let
        ( indicatorColor, label, shouldPulse ) =
            if online then
                ( colors.success, "ONLINE", True )
            else
                ( colors.textMuted, "OFFLINE", False )
    in
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.5rem"
        , style "padding" "0.375rem 0.75rem"
        , style "background-color" colors.bgSurface
        , style "border-radius" "2px"
        ]
        [ div
            [ style "width" "6px"
            , style "height" "6px"
            , style "background-color" indicatorColor
            , style "border-radius" "50%"
            , style "box-shadow" ("0 0 8px " ++ indicatorColor)
            , style "animation" (if shouldPulse then "statusPulse 2s infinite" else "none")
            ]
            []
        , span
            [ style "font-family" fontMono
            , style "font-size" "0.5625rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.1em"
            , style "color" indicatorColor
            ]
            [ text label ]
        ]


navDivider : Html msg
navDivider =
    div
        [ style "width" "1px"
        , style "background-color" colors.border
        , style "margin" "0.5rem 0.25rem"
        , style "align-self" "stretch"
        ]
        []


navLink : String -> Route -> Route -> (Route -> msg) -> Html msg
navLink label route currentRoute onNavigate =
    let
        isActive =
            case ( route, currentRoute ) of
                ( WelcomeRoute, WelcomeRoute ) -> True
                ( DashboardRoute, DashboardRoute ) -> True
                ( MessagesRoute _ _, MessagesRoute _ _ ) -> True
                ( LogsRoute, LogsRoute ) -> True
                ( CronJobsRoute, CronJobsRoute ) -> True
                ( SettingsRoute, SettingsRoute ) -> True
                ( CapabilitiesRoute, CapabilitiesRoute ) -> True
                ( ProjectsRoute, ProjectsRoute ) -> True
                ( ProjectsRoute, ProjectDetailRoute _ ) -> True
                ( LiveBoardRoute, LiveBoardRoute ) -> True
                ( ChatRoute _, ChatRoute _ ) -> True
                _ -> False
    in
    button
        [ onClick (onNavigate route)
        , style "background" "transparent"
        , style "color" (if isActive then colors.accent else colors.textSecondary)
        , style "border" "none"
        , style "border-bottom" (if isActive then ("2px solid " ++ colors.accent) else "2px solid transparent")
        , style "padding" "0 clamp(0.75rem, 4vw, 1.5rem)"
        , style "cursor" "pointer"
        , style "font-family" fontBody
        , style "font-size" "clamp(0.6875rem, 2.8vw, 0.8125rem)"
        , style "font-weight" "500"
        , style "letter-spacing" "0.02em"
        , style "text-transform" "uppercase"
        -- Nav tabs should feel snappy; disable the global button hover animation here
        , style "transition" "none"
        , style "transform" "none"
        , style "filter" "none"
        , style "position" "relative"
        , style "white-space" "nowrap"
        ]
        [ text label
        , if isActive then
            div
                [ style "position" "absolute"
                , style "bottom" "-1px"
                , style "left" "0"
                , style "right" "0"
                , style "height" "1px"
                , style "background" colors.accent
                , style "box-shadow" ("0 0 12px " ++ colors.accent)
                ]
                []
          else
            text ""
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- PAGE LAYOUT
-- ═══════════════════════════════════════════════════════════════════════════


pageHeader : String -> List (Html msg) -> Html msg
pageHeader title actions =
    div
        [ style "display" "flex"
        , style "justify-content" "space-between"
        , style "align-items" "center"
        , style "flex-wrap" "wrap"
        , style "gap" "1rem"
        , style "margin-bottom" "clamp(1.25rem, 4vw, 2rem)"
        , style "padding-top" "clamp(1.25rem, 4vw, 2rem)"
        , style "padding-bottom" "clamp(1rem, 3vw, 1.5rem)"
        , style "border-bottom" ("1px solid " ++ colors.border)
        ]
        [ div [ style "display" "flex", style "align-items" "baseline", style "gap" "1rem" ]
            [ h2
                [ style "font-family" fontDisplay
                , style "font-size" "clamp(1.25rem, 5vw, 1.75rem)"
                , style "font-weight" "600"
                , style "letter-spacing" "0.02em"
                , style "text-transform" "uppercase"
                , style "margin" "0"
                , style "color" colors.textPrimary
                ]
                [ text title ]
            , div
                [ style "width" "32px"
                , style "height" "2px"
                , style "background" ("linear-gradient(90deg, " ++ colors.accent ++ ", transparent)")
                ]
                []
            ]
        , div
            [ style "display" "flex"
            , style "gap" "0.75rem"
            , style "flex-wrap" "wrap"
            ]
            actions
        ]


sectionHeader : String -> Html msg
sectionHeader title =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.75rem"
        , style "margin-bottom" "1rem"
        ]
        [ div
            [ style "width" "3px"
            , style "height" "16px"
            , style "background-color" colors.accent
            ]
            []
        , h3
            [ style "font-family" fontMono
            , style "font-size" "0.6875rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.12em"
            , style "text-transform" "uppercase"
            , style "color" colors.textMuted
            , style "margin" "0"
            ]
            [ text title ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- CARDS / PANELS
-- ═══════════════════════════════════════════════════════════════════════════


card : List (Attribute msg) -> List (Html msg) -> Html msg
card attrs content =
    div
        ([ style "background-color" colors.bgTertiary
         , style "border" ("1px solid " ++ colors.border)
         , style "border-radius" "4px"
         , style "padding" "1.5rem"
         , style "position" "relative"
         , style "overflow" "hidden"
         ]
            ++ attrs
        )
        ([ -- Top accent line
           div
            [ style "position" "absolute"
            , style "top" "0"
            , style "left" "0"
            , style "right" "0"
            , style "height" "1px"
            , style "background" ("linear-gradient(90deg, " ++ colors.accent ++ ", transparent 60%)")
            , style "opacity" "0.5"
            ]
            []
         ] ++ content)


cardWithHeader : String -> List (Attribute msg) -> List (Html msg) -> Html msg
cardWithHeader title attrs content =
    card attrs
        (cardHeader title :: content)


cardHeader : String -> Html msg
cardHeader title =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.625rem"
        , style "margin-bottom" "1.25rem"
        , style "padding-bottom" "0.75rem"
        , style "border-bottom" ("1px solid " ++ colors.border)
        ]
        [ div
            [ style "width" "4px"
            , style "height" "4px"
            , style "background-color" colors.accent
            , style "box-shadow" ("0 0 6px " ++ colors.accent)
            ]
            []
        , h3
            [ style "font-family" fontMono
            , style "font-size" "0.6875rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.12em"
            , style "text-transform" "uppercase"
            , style "color" colors.textSecondary
            , style "margin" "0"
            ]
            [ text title ]
        ]


{-| Subtle alternating shading helper for list items
-}
zebraOverlay : Int -> List (Attribute msg)
zebraOverlay index =
    if modBy 2 index == 1 then
        [ style "background-image" "linear-gradient(rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.025))" ]
    else
        []


-- ═══════════════════════════════════════════════════════════════════════════
-- GRID LAYOUTS
-- ═══════════════════════════════════════════════════════════════════════════


gridTwo : List (Html msg) -> Html msg
gridTwo content =
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fit, minmax(320px, 1fr))"
        , style "gap" "1.5rem"
        ]
        content


-- ═══════════════════════════════════════════════════════════════════════════
-- BUTTONS
-- ═══════════════════════════════════════════════════════════════════════════


button_ : List (Attribute msg) -> String -> Html msg
button_ attrs label =
    button
        ([ style "background-color" colors.bgSurface
         , style "color" colors.textSecondary
         , style "border" ("1px solid " ++ colors.border)
         , style "padding" "0.5rem 1.25rem"
         , style "border-radius" "2px"
         , style "cursor" "pointer"
         , style "font-family" fontMono
         , style "font-size" "0.75rem"
         , style "font-weight" "500"
         , style "letter-spacing" "0.05em"
         , style "text-transform" "uppercase"
         , style "transition" "all 0.15s ease"
         ]
            ++ attrs
        )
        [ text label ]


primaryButton : List (Attribute msg) -> String -> Html msg
primaryButton attrs label =
    button
        ([ style "background-color" colors.accent
         , style "color" colors.bgPrimary
         , style "border" "none"
         , style "padding" "0.5rem 1.25rem"
         , style "border-radius" "2px"
         , style "cursor" "pointer"
         , style "font-family" fontMono
         , style "font-size" "0.75rem"
         , style "font-weight" "600"
         , style "letter-spacing" "0.05em"
         , style "text-transform" "uppercase"
         , style "transition" "all 0.15s ease"
         , style "box-shadow" ("0 0 20px " ++ colors.accentGlow)
         ]
            ++ attrs
        )
        [ text label ]


iconButton : String -> List (Attribute msg) -> Html msg
iconButton icon attrs =
    button
        ([ style "background-color" "transparent"
         , style "color" colors.textMuted
         , style "border" "none"
         , style "padding" "0.5rem"
         , style "border-radius" "2px"
         , style "cursor" "pointer"
         , style "font-size" "1.125rem"
         , style "line-height" "1"
         , style "transition" "all 0.15s ease"
         ]
            ++ attrs
        )
        [ text icon ]


-- ═══════════════════════════════════════════════════════════════════════════
-- BADGES
-- ═══════════════════════════════════════════════════════════════════════════


badge : String -> String -> Html msg
badge color label =
    span
        [ style "background-color" color
        , style "color" colors.bgPrimary
        , style "font-family" fontMono
        , style "font-size" "0.5625rem"
        , style "font-weight" "700"
        , style "padding" "0.25rem 0.5rem"
        , style "border-radius" "1px"
        , style "text-transform" "uppercase"
        , style "letter-spacing" "0.08em"
        ]
        [ text label ]


pillBadge : String -> String -> String -> Html msg
pillBadge bgColor textColor label =
    span
        [ style "background-color" bgColor
        , style "color" textColor
        , style "font-family" fontMono
        , style "font-size" "0.625rem"
        , style "font-weight" "600"
        , style "padding" "0.25rem 0.625rem"
        , style "border-radius" "2px"
        , style "letter-spacing" "0.05em"
        ]
        [ text label ]


statusBadge : String -> Html msg
statusBadge status =
    let
        (bgColor, textColor, label) =
            case status of
                "pending" -> (colors.borderLight, colors.textMuted, "PENDING")
                "running" -> (colors.warningDim, colors.warning, "RUNNING")
                "completed" -> (colors.successDim, colors.success, "DONE")
                "failed" -> (colors.errorDim, colors.error, "FAILED")
                "sent" -> (colors.successDim, colors.success, "SENT")
                _ -> (colors.borderLight, colors.textMuted, String.toUpper status)
    in
    pillBadge bgColor textColor label


-- ═══════════════════════════════════════════════════════════════════════════
-- STATS
-- ═══════════════════════════════════════════════════════════════════════════


statCard : String -> String -> String -> Html msg
statCard label value accent =
    div
        [ style "background-color" colors.bgTertiary
        , style "border" ("1px solid " ++ colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        , style "position" "relative"
        , style "overflow" "hidden"
        ]
        [ -- Accent corner
          div
            [ style "position" "absolute"
            , style "top" "0"
            , style "left" "0"
            , style "width" "40px"
            , style "height" "40px"
            , style "background" ("linear-gradient(135deg, " ++ accent ++ " 0%, transparent 70%)")
            , style "opacity" "0.15"
            ]
            []
        , div
            [ style "position" "relative"
            ]
            [ div
                [ style "font-family" fontMono
                , style "font-size" "0.625rem"
                , style "font-weight" "600"
                , style "letter-spacing" "0.12em"
                , style "text-transform" "uppercase"
                , style "color" colors.textMuted
                , style "margin-bottom" "0.625rem"
                ]
                [ text label ]
            , div
                [ style "font-family" fontDisplay
                , style "font-size" "2.5rem"
                , style "font-weight" "600"
                , style "color" accent
                , style "line-height" "1"
                , style "letter-spacing" "-0.02em"
                ]
                [ text value ]
            ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- EMPTY STATES
-- ═══════════════════════════════════════════════════════════════════════════


emptyState : String -> Html msg
emptyState message =
    emptyStateWithIcon "—" message


emptyStateWithIcon : String -> String -> Html msg
emptyStateWithIcon icon message =
    div
        [ style "text-align" "center"
        , style "padding" "3rem 2rem"
        ]
        [ div
            [ style "font-size" "2rem"
            , style "margin-bottom" "1rem"
            , style "opacity" "0.3"
            , style "color" colors.textMuted
            ]
            [ text icon ]
        , div
            [ style "font-family" fontMono
            , style "font-size" "0.75rem"
            , style "letter-spacing" "0.05em"
            , style "color" colors.textMuted
            ]
            [ text message ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- LOADING
-- ═══════════════════════════════════════════════════════════════════════════


loadingSpinner : Html msg
loadingSpinner =
    div
        [ style "display" "flex"
        , style "justify-content" "center"
        , style "align-items" "center"
        , style "padding" "3rem"
        ]
        [ div
            [ style "width" "32px"
            , style "height" "32px"
            , style "border" ("2px solid " ++ colors.border)
            , style "border-top-color" colors.accent
            , style "border-radius" "50%"
            , style "animation" "spin 0.8s linear infinite"
            ]
            []
        ]


loadingText : String -> Html msg
loadingText message =
    div
        [ style "display" "flex"
        , style "flex-direction" "column"
        , style "align-items" "center"
        , style "padding" "3rem"
        , style "gap" "1rem"
        ]
        [ div
            [ style "width" "32px"
            , style "height" "32px"
            , style "border" ("2px solid " ++ colors.border)
            , style "border-top-color" colors.accent
            , style "border-radius" "50%"
            , style "animation" "spin 0.8s linear infinite"
            ]
            []
        , span
            [ style "font-family" fontMono
            , style "font-size" "0.75rem"
            , style "letter-spacing" "0.05em"
            , style "color" colors.textMuted
            ]
            [ text message ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- UTILITY FUNCTIONS
-- ═══════════════════════════════════════════════════════════════════════════


formatTime : String -> String
formatTime ts =
    String.slice 11 19 ts


formatDateTime : String -> String
formatDateTime ts =
    let
        date = String.slice 8 10 ts
        month = String.slice 5 7 ts
        time = String.slice 11 16 ts

        monthName =
            case month of
                "01" -> "Jan"
                "02" -> "Feb"
                "03" -> "Mar"
                "04" -> "Apr"
                "05" -> "May"
                "06" -> "Jun"
                "07" -> "Jul"
                "08" -> "Aug"
                "09" -> "Sep"
                "10" -> "Oct"
                "11" -> "Nov"
                "12" -> "Dec"
                _ -> month
    in
    date ++ " " ++ monthName ++ " " ++ time


truncateText : Int -> String -> String
truncateText maxLen str =
    if String.length str > maxLen then
        String.left maxLen str ++ "..."
    else
        str


-- ═══════════════════════════════════════════════════════════════════════════
-- PAGE INFO BAR (shared)
-- ═══════════════════════════════════════════════════════════════════════════


type alias PageInfoConfig =
    { page : Int
    , pageSize : Int
    , total : Int
    , totalPages : Int
    }


pageInfo : PageInfoConfig -> Html msg
pageInfo config =
    let
        startNum = config.page * config.pageSize + 1
        endNum = Basics.min ((config.page + 1) * config.pageSize) config.total
    in
    div
        [ style "display" "flex"
        , style "justify-content" "space-between"
        , style "align-items" "center"
        , style "flex-wrap" "wrap"
        , style "gap" "0.75rem"
        , style "margin-bottom" "1rem"
        , style "padding" "0.75rem 1rem"
        , style "background-color" colors.bgTertiary
        , style "border" ("1px solid " ++ colors.border)
        , style "border-radius" "4px"
        ]
        [ monoLabel ("SHOWING " ++ String.fromInt startNum ++ "–" ++ String.fromInt endNum ++ " OF " ++ String.fromInt config.total)
        , span
            [ style "font-family" fontMono
            , style "font-size" "0.6875rem"
            , style "color" colors.textSecondary
            , style "letter-spacing" "0.05em"
            ]
            [ text ("PAGE " ++ String.fromInt (config.page + 1) ++ " / " ++ String.fromInt config.totalPages) ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- MINI STAT (for grids of small stats)
-- ═══════════════════════════════════════════════════════════════════════════


miniStat : String -> Int -> String -> Html msg
miniStat label count color =
    div
        [ style "padding" "1rem"
        , style "background-color" colors.bgSurface
        , style "border-radius" "4px"
        , style "text-align" "center"
        ]
        [ div
            [ style "font-family" fontDisplay
            , style "font-size" "1.75rem"
            , style "font-weight" "600"
            , style "color" color
            , style "line-height" "1"
            ]
            [ text (String.fromInt count) ]
        , monoLabel label
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- STYLE HELPERS (reduce repetition)
-- ═══════════════════════════════════════════════════════════════════════════


{-| Small uppercase mono label (commonly used for stats, headers)
-}
monoLabel : String -> Html msg
monoLabel content =
    span
        [ style "font-family" fontMono
        , style "font-size" "0.5625rem"
        , style "color" colors.textMuted
        , style "text-transform" "uppercase"
        , style "letter-spacing" "0.1em"
        ]
        [ text content ]


{-| Flex row with gap
-}
row : String -> List (Html msg) -> Html msg
row gap content =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "gap" gap
        ]
        content


{-| Flex row with space-between
-}
rowBetween : List (Html msg) -> Html msg
rowBetween content =
    div
        [ style "display" "flex"
        , style "align-items" "center"
        , style "justify-content" "space-between"
        ]
        content


{-| Flex column with gap
-}
col : String -> List (Html msg) -> Html msg
col gap content =
    div
        [ style "display" "flex"
        , style "flex-direction" "column"
        , style "gap" gap
        ]
        content


{-| Status dot with glow
-}
statusDot : String -> Bool -> Html msg
statusDot color shouldPulse =
    div
        [ style "width" "8px"
        , style "height" "8px"
        , style "background-color" color
        , style "border-radius" "50%"
        , style "box-shadow" ("0 0 12px " ++ color)
        , style "animation" (if shouldPulse then "statusPulse 2s infinite" else "none")
        ]
        []


{-| Table header cell
-}
tableHeader : String -> Html msg
tableHeader label =
    span
        [ style "font-family" fontMono
        , style "font-size" "0.5625rem"
        , style "font-weight" "700"
        , style "text-transform" "uppercase"
        , style "letter-spacing" "0.12em"
        , style "color" colors.textMuted
        ]
        [ text label ]


{-| Back button with arrow
-}
backButton : msg -> Html msg
backButton onBack =
    button
        [ onClick onBack
        , style "background-color" "transparent"
        , style "color" colors.textSecondary
        , style "border" ("1px solid " ++ colors.border)
        , style "padding" "0.5rem 1.25rem"
        , style "border-radius" "2px"
        , style "cursor" "pointer"
        , style "font-family" fontMono
        , style "font-size" "0.75rem"
        , style "font-weight" "500"
        , style "letter-spacing" "0.05em"
        , style "text-transform" "uppercase"
        , style "display" "inline-flex"
        , style "align-items" "center"
        , style "gap" "0.5rem"
        ]
        [ span [ style "font-size" "0.875rem" ] [ text "←" ]
        , text "Back"
        ]


{-| Role/direction badge (USER/BOT)
-}
roleBadge : String -> String -> Html msg
roleBadge label color =
    let
        bgColor =
            if color == colors.accent then
                colors.accentDim
            else if color == colors.success then
                colors.successDim
            else
                colors.borderLight
    in
    span
        [ style "font-family" fontMono
        , style "font-weight" "700"
        , style "font-size" "0.625rem"
        , style "letter-spacing" "0.1em"
        , style "color" color
        , style "padding" "0.25rem 0.5rem"
        , style "background-color" bgColor
        , style "border-radius" "2px"
        ]
        [ text label ]


{-| Media type badge with icon
-}
mediaTypeBadge : String -> String -> Html msg
mediaTypeBadge icon label =
    span
        [ style "display" "inline-flex"
        , style "align-items" "center"
        , style "gap" "0.375rem"
        , style "padding" "0.25rem 0.5rem"
        , style "background-color" colors.bgSurface
        , style "border" ("1px solid " ++ colors.border)
        , style "border-radius" "2px"
        , style "font-family" fontMono
        , style "font-size" "0.625rem"
        , style "color" colors.textSecondary
        , style "letter-spacing" "0.05em"
        ]
        [ span [ style "color" colors.accent ] [ text icon ]
        , text (String.toUpper label)
        ]


{-| Timestamp display
-}
timestamp : String -> Html msg
timestamp ts =
    span
        [ style "font-family" fontMono
        , style "color" colors.textMuted
        , style "font-size" "0.6875rem"
        , style "letter-spacing" "0.02em"
        ]
        [ text (formatDateTime ts) ]


{-| Item with left border accent
-}
accentedItem : String -> List (Attribute msg) -> List (Html msg) -> Html msg
accentedItem borderColor attrs content =
    div
        ([ style "background-color" colors.bgSurface
         , style "border-radius" "4px"
         , style "padding" "1rem 1.25rem"
         , style "border-left" ("3px solid " ++ borderColor)
         ]
            ++ attrs
        )
        content


-- ═══════════════════════════════════════════════════════════════════════════
-- FORM HELPERS
-- ═══════════════════════════════════════════════════════════════════════════


formField : String -> Html msg -> Html msg
formField label input_ =
    div []
        [ div
            [ style "font-family" fontMono
            , style "font-size" "0.625rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.1em"
            , style "text-transform" "uppercase"
            , style "color" colors.textMuted
            , style "margin-bottom" "0.5rem"
            ]
            [ text label ]
        , input_
        ]


inputField : String -> (String -> msg) -> String -> Html msg
inputField val onChange placeholderText =
    input
        [ value val
        , onInput onChange
        , placeholder placeholderText
        , style "width" "100%"
        , style "background-color" colors.bgPrimary
        , style "color" colors.textPrimary
        , style "border" ("1px solid " ++ colors.border)
        , style "border-radius" "2px"
        , style "padding" "0.5rem 0.75rem"
        , style "font-family" fontBody
        , style "font-size" "0.875rem"
        , style "box-sizing" "border-box"
        ]
        []


textareaField : String -> (String -> msg) -> String -> Html msg
textareaField val onChange placeholderText =
    textarea
        [ value val
        , onInput onChange
        , placeholder placeholderText
        , style "width" "100%"
        , style "min-height" "80px"
        , style "background-color" colors.bgPrimary
        , style "color" colors.textPrimary
        , style "border" ("1px solid " ++ colors.border)
        , style "border-radius" "2px"
        , style "padding" "0.5rem 0.75rem"
        , style "font-family" fontBody
        , style "font-size" "0.875rem"
        , style "resize" "vertical"
        , style "box-sizing" "border-box"
        ]
        []


selectField : String -> (String -> msg) -> List ( String, String ) -> Html msg
selectField val onChange options =
    select
        [ value val
        , onInput onChange
        , style "width" "100%"
        , style "background-color" colors.bgPrimary
        , style "color" colors.textPrimary
        , style "border" ("1px solid " ++ colors.border)
        , style "border-radius" "2px"
        , style "padding" "0.5rem 0.75rem"
        , style "font-family" fontBody
        , style "font-size" "0.875rem"
        , style "box-sizing" "border-box"
        ]
        (List.map (\( v, l ) -> option [ value v, selected (v == val) ] [ text l ]) options)


tagChip : String -> Html msg
tagChip tag =
    span
        [ style "font-family" fontMono
        , style "font-size" "0.5625rem"
        , style "color" colors.textSecondary
        , style "padding" "0.1875rem 0.5rem"
        , style "background-color" colors.borderLight
        , style "border-radius" "2px"
        , style "letter-spacing" "0.03em"
        ]
        [ text tag ]


docTypeBadge : String -> Html msg
docTypeBadge docType =
    let
        ( bgColor, textColor, label ) =
            case docType of
                "plan" -> ( "rgba(96, 165, 250, 0.12)", "#60a5fa", "PLAN" )
                "specification" -> ( "rgba(167, 139, 250, 0.12)", "#a78bfa", "SPEC" )
                "notes" -> ( colors.borderLight, colors.textSecondary, "NOTES" )
                "code" -> ( "rgba(0, 212, 170, 0.12)", colors.accent, "CODE" )
                _ -> ( colors.borderLight, colors.textMuted, String.toUpper docType )
    in
    pillBadge bgColor textColor label


taskStatusColor : String -> String
taskStatusColor status =
    case status of
        "todo" -> colors.textMuted
        "in_progress" -> colors.warning
        "ready_for_review" -> colors.accent
        "under_review" -> "#a78bfa"
        "done" -> colors.success
        "blocked" -> colors.error
        "abandoned" -> colors.textMuted
        _ -> colors.border
