module Pages.Logs exposing (view)

import Components.Pagination
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


view : RemoteData LogsPage -> String -> (String -> msg) -> msg -> (Int -> msg) -> msg -> Html msg
view logsPage searchTerm onSearchChange onSearchSubmit onPageChange onRefresh =
    div []
        [ UI.pageHeader "System Logs"
            [ UI.button_ [ onClick onRefresh ] "Refresh" ]
        , viewSearchBar searchTerm onSearchChange onSearchSubmit
        , case logsPage of
            Loading ->
                UI.loadingSpinner

            Failure err ->
                UI.card [] [ text ("Error: " ++ err) ]

            NotAsked ->
                UI.loadingText "Loading logs..."

            Success page ->
                if List.isEmpty page.entries && String.isEmpty searchTerm then
                    UI.card []
                        [ UI.emptyStateWithIcon "—" "No logs recorded yet" ]
                else
                let
                    paginationConfig =
                        { page = page.page
                        , totalPages = page.totalPages
                        , onPageChange = onPageChange
                        }
                in
                    div []
                        [ viewLogStats page.entries
                    , div
                        [ style "display" "flex"
                        , style "justify-content" "space-between"
                        , style "align-items" "center"
                        , style "gap" "1rem"
                        , style "flex-wrap" "wrap"
                        ]
                        [ UI.pageInfo
                            { page = page.page
                            , pageSize = page.pageSize
                            , total = page.total
                            , totalPages = page.totalPages
                            }
                        , Components.Pagination.viewCompact paginationConfig
                        ]
                        , viewLogTable page.entries
                        , if page.totalPages > 1 then
                        Components.Pagination.view paginationConfig
                          else
                            text ""
                        ]
        ]


viewSearchBar : String -> (String -> msg) -> msg -> Html msg
viewSearchBar searchTerm onSearchChange onSearchSubmit =
    Html.form
        [ onSubmit onSearchSubmit
        , style "display" "flex"
        , style "flex-wrap" "wrap"
        , style "gap" "0.75rem"
        , style "margin-bottom" "2rem"
        ]
        [ input
            [ type_ "text"
            , placeholder "Search logs..."
            , value searchTerm
            , onInput onSearchChange
            , style "flex" "1"
            , style "min-width" "14rem"
            , style "padding" "0.75rem 1rem"
            , style "background-color" UI.colors.bgTertiary
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "4px"
            , style "color" UI.colors.textPrimary
            , style "font-family" UI.fontMono
            , style "font-size" "0.8125rem"
            ]
            []
        , UI.primaryButton [ type_ "submit" ] "Search"
        ]


viewLogStats : List LogEntry -> Html msg
viewLogStats logList =
    let
        countByLevel level =
            List.filter (\e -> e.level == level) logList |> List.length
    in
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fit, minmax(140px, 1fr))"
        , style "gap" "1rem"
        , style "margin-bottom" "2rem"
        ]
        [ logStatCard "Error" (countByLevel "error") UI.colors.error
        , logStatCard "Warn" (countByLevel "warn") UI.colors.warning
        , logStatCard "Info" (countByLevel "info") UI.colors.success
        , logStatCard "Debug" (countByLevel "debug") UI.colors.textMuted
        ]


logStatCard : String -> Int -> String -> Html msg
logStatCard label count color =
    div
        [ style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        , style "text-align" "center"
        , style "position" "relative"
        , style "overflow" "hidden"
        ]
        [ div
            [ style "position" "absolute"
            , style "top" "0"
            , style "left" "0"
            , style "right" "0"
            , style "height" "2px"
            , style "background-color" color
            , style "opacity" "0.6"
            ]
            []
        , div
            [ style "font-family" UI.fontDisplay
            , style "font-size" "2rem"
            , style "font-weight" "700"
            , style "color" color
            , style "line-height" "1"
            ]
            [ text (String.fromInt count) ]
        , div [ style "margin-top" "0.5rem" ] [ UI.monoLabel label ]
        ]


viewLogTable : List LogEntry -> Html msg
viewLogTable logList =
    div
        [ style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "overflow" "hidden"
        ]
        [ div
            [ style "overflow-x" "auto"
            , style "-webkit-overflow-scrolling" "touch"
            ]
            [ div
                [ style "min-width" "720px" ]
                [ div
                    [ style "display" "grid"
                    , style "grid-template-columns" "minmax(80px, 100px) minmax(60px, 80px) minmax(100px, 140px) 1fr"
                    , style "gap" "1rem"
                    , style "padding" "0.75rem 1rem"
                    , style "background-color" UI.colors.bgSurface
                    , style "border-bottom" ("1px solid " ++ UI.colors.border)
                    ]
                    [ UI.tableHeader "Time"
                    , UI.tableHeader "Level"
                    , UI.tableHeader "Component"
                    , UI.tableHeader "Message"
                    ]
                , div
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.8125rem"
                    , style "max-height" "600px"
                    , style "overflow-y" "auto"
                    ]
                    (if List.isEmpty logList then
                        [ div
                            [ style "padding" "2.5rem"
                            , style "text-align" "center"
                            , style "color" UI.colors.textMuted
                            , style "font-size" "0.75rem"
                            , style "letter-spacing" "0.05em"
                            ]
                            [ text "No matching logs found" ]
                        ]
                     else
                        List.indexedMap viewLogEntry logList
                    )
                ]
            ]
        ]


viewLogEntry : Int -> LogEntry -> Html msg
viewLogEntry index entry =
    let
        levelColor =
            case entry.level of
                "error" -> UI.colors.error
                "warn" -> UI.colors.warning
                "info" -> UI.colors.success
                "debug" -> UI.colors.textMuted
                _ -> UI.colors.textSecondary
    in
    div
        ([ style "display" "grid"
         , style "grid-template-columns" "minmax(80px, 100px) minmax(60px, 80px) minmax(100px, 140px) 1fr"
         , style "gap" "1rem"
         , style "padding" "0.625rem 1rem"
         , style "border-bottom" ("1px solid " ++ UI.colors.borderLight)
         , style "align-items" "baseline"
         ]
            ++ UI.zebraOverlay index
        )
        [ span [ style "color" UI.colors.textMuted, style "font-size" "0.75rem" ]
            [ text (UI.formatTime entry.timestamp) ]
        , span
            [ style "display" "inline-flex"
            , style "align-items" "center"
            , style "justify-content" "center"
            , style "color" levelColor
            , style "font-weight" "700"
            , style "text-transform" "uppercase"
            , style "font-size" "0.5625rem"
            , style "letter-spacing" "0.08em"
            ]
            [ text entry.level ]
        , span
            [ style "color" UI.colors.accent
            , style "font-size" "0.75rem"
            , style "overflow" "hidden"
            , style "text-overflow" "ellipsis"
            , style "white-space" "nowrap"
            ]
            [ text entry.component ]
        , span
            [ style "color" UI.colors.textSecondary
            , style "word-break" "break-word"
            , style "line-height" "1.5"
            , style "font-size" "0.8125rem"
            ]
            [ text entry.message ]
        ]
