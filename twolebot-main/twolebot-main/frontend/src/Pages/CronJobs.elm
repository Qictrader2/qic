module Pages.CronJobs exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


view :
    RemoteData (List CronJob)
    -> RemoteData CronStatus
    -> msg
    -> (String -> msg)
    -> (String -> msg)
    -> (String -> msg)
    -> Html msg
view cronJobs cronStatus onRefresh onPause onResume onCancel =
    div []
        [ UI.pageHeader "Scheduled Jobs"
            [ UI.button_ [ onClick onRefresh ] "Refresh" ]
        , case ( cronJobs, cronStatus ) of
            ( Loading, _ ) ->
                UI.loadingSpinner

            ( _, Loading ) ->
                UI.loadingSpinner

            ( Failure err, _ ) ->
                UI.card [] [ text ("Error: " ++ err) ]

            ( _, Failure err ) ->
                UI.card [] [ text ("Error: " ++ err) ]

            ( NotAsked, _ ) ->
                UI.loadingText "Loading cron jobs..."

            ( _, NotAsked ) ->
                UI.loadingText "Loading cron status..."

            ( Success jobs, Success status ) ->
                UI.col "2rem"
                    [ viewStatusCards status
                    , viewJobsList jobs onPause onResume onCancel
                    ]
        ]


viewStatusCards : CronStatus -> Html msg
viewStatusCards status =
    div
        [ style "display" "grid"
        , style "grid-template-columns" "repeat(auto-fit, minmax(200px, 1fr))"
        , style "gap" "1rem"
        ]
        [ UI.statCard "Active Jobs" (String.fromInt status.activeJobs) UI.colors.success
        , UI.statCard "Paused" (String.fromInt status.pausedJobs) UI.colors.warning
        , UI.statCard "Waiting" (String.fromInt status.waitingExecutions) UI.colors.accent
        ]


viewJobsList : List CronJob -> (String -> msg) -> (String -> msg) -> (String -> msg) -> Html msg
viewJobsList jobs onPause onResume onCancel =
    UI.cardWithHeader ("Jobs (" ++ String.fromInt (List.length jobs) ++ ")") []
        [ if List.isEmpty jobs then
            UI.emptyStateWithIcon "---" "No scheduled jobs"
          else
            UI.col "0.75rem" (List.indexedMap (viewJobItem onPause onResume onCancel) jobs)
        ]


viewJobItem : (String -> msg) -> (String -> msg) -> (String -> msg) -> Int -> CronJob -> Html msg
viewJobItem onPause onResume onCancel index job =
    let
        borderColor =
            case job.status of
                "active" ->
                    UI.colors.success

                "paused" ->
                    UI.colors.warning

                "cancelled" ->
                    UI.colors.error

                _ ->
                    UI.colors.textMuted

        jobName =
            Maybe.withDefault ("Job " ++ String.left 8 job.id) job.name
    in
    UI.accentedItem borderColor (UI.zebraOverlay index)
        [ -- Header row
          div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "justify-content" "space-between"
            , style "margin-bottom" "0.75rem"
            , style "flex-wrap" "wrap"
            , style "gap" "0.75rem"
            ]
            [ UI.row "0.75rem"
                [ viewStatusBadge job.status
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-weight" "600"
                    , style "font-size" "0.875rem"
                    , style "color" UI.colors.textPrimary
                    ]
                    [ text jobName ]
                ]
            , UI.row "0.5rem"
                [ viewActionButton job onPause onResume
                , button
                    [ onClick (onCancel job.id)
                    , style "background-color" "transparent"
                    , style "color" UI.colors.error
                    , style "border" ("1px solid " ++ UI.colors.error)
                    , style "padding" "0.25rem 0.75rem"
                    , style "border-radius" "2px"
                    , style "cursor" "pointer"
                    , style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "font-weight" "600"
                    , style "letter-spacing" "0.05em"
                    , style "text-transform" "uppercase"
                    ]
                    [ text "Cancel" ]
                ]
            ]
        , -- Details grid
          div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(180px, 1fr))"
            , style "gap" "1rem"
            , style "margin-top" "0.75rem"
            ]
            [ viewDetailItem "SCHEDULE" job.schedule
            , viewDetailItem "DEFERRABLE" (if job.deferrable then "Yes" else "No")
            , viewDetailItem "NEXT RUN" (Maybe.withDefault "---" (Maybe.map UI.formatDateTime job.nextRun))
            , viewDetailItem "LAST RUN" (Maybe.withDefault "---" (Maybe.map UI.formatDateTime job.lastRun))
            ]
        , -- Footer
          div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-top" "1rem"
            , style "padding-top" "0.75rem"
            , style "border-top" ("1px solid " ++ UI.colors.border)
            ]
            [ span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "color" UI.colors.textMuted
                , style "letter-spacing" "0.05em"
                ]
                [ text ("ID: " ++ job.id) ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "color" UI.colors.textMuted
                , style "letter-spacing" "0.05em"
                ]
                [ text ("Created " ++ UI.formatDateTime job.createdAt) ]
            ]
        ]


viewStatusBadge : String -> Html msg
viewStatusBadge status =
    let
        ( bgColor, textColor, label ) =
            case status of
                "active" ->
                    ( UI.colors.successDim, UI.colors.success, "ACTIVE" )

                "paused" ->
                    ( UI.colors.warningDim, UI.colors.warning, "PAUSED" )

                "cancelled" ->
                    ( UI.colors.errorDim, UI.colors.error, "CANCELLED" )

                _ ->
                    ( UI.colors.borderLight, UI.colors.textMuted, String.toUpper status )
    in
    UI.pillBadge bgColor textColor label


viewActionButton : CronJob -> (String -> msg) -> (String -> msg) -> Html msg
viewActionButton job onPause onResume =
    case job.status of
        "active" ->
            button
                [ onClick (onPause job.id)
                , style "background-color" "transparent"
                , style "color" UI.colors.warning
                , style "border" ("1px solid " ++ UI.colors.warning)
                , style "padding" "0.25rem 0.75rem"
                , style "border-radius" "2px"
                , style "cursor" "pointer"
                , style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "font-weight" "600"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "Pause" ]

        "paused" ->
            button
                [ onClick (onResume job.id)
                , style "background-color" "transparent"
                , style "color" UI.colors.success
                , style "border" ("1px solid " ++ UI.colors.success)
                , style "padding" "0.25rem 0.75rem"
                , style "border-radius" "2px"
                , style "cursor" "pointer"
                , style "font-family" UI.fontMono
                , style "font-size" "0.625rem"
                , style "font-weight" "600"
                , style "letter-spacing" "0.05em"
                , style "text-transform" "uppercase"
                ]
                [ text "Resume" ]

        _ ->
            text ""


viewDetailItem : String -> String -> Html msg
viewDetailItem label value =
    div []
        [ div
            [ style "font-family" UI.fontMono
            , style "font-size" "0.5625rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.1em"
            , style "color" UI.colors.textMuted
            , style "margin-bottom" "0.25rem"
            ]
            [ text label ]
        , div
            [ style "font-family" UI.fontMono
            , style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            ]
            [ text value ]
        ]
