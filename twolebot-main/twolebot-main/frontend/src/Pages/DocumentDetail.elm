module Pages.DocumentDetail exposing (view)

import Components.Comments
import Components.Markdown
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (..)
import UI


view :
    RemoteData WorkDocument
    -> RemoteData (List WorkComment)
    -> CommentForm
    -> Maybe Int
    -> Maybe Int
    -> List Int
    -> Bool
    -> VoiceState
    -> { onNavigate : Route -> msg
       , onRefresh : msg
       , onCommentChange : String -> msg
       , onSubmitComment : msg
       , onSubmitReply : Int -> msg
       , onStartReply : Int -> msg
       , onCancelReply : msg
       , onStartEdit : Int -> msg
       , onSaveEdit : Int -> msg
       , onCancelEdit : msg
       , onToggleCommentCollapse : Int -> msg
       , onVoiceStartRecording : VoiceMode -> msg
       , onVoiceStopRecording : msg
       , onVoiceReset : msg
       , onVoiceApplyToComment : msg
       , onVoiceEditComment : Int -> msg
       }
    -> Html msg
view document comments commentForm replyingToCommentId editingCommentId collapsedComments isBusy voiceState config =
    case document of
        NotAsked ->
            UI.emptyState "Loading..."

        Loading ->
            UI.loadingSpinner

        Failure err ->
            UI.emptyState ("Error: " ++ err)

        Success doc ->
            div []
                [ div
                    [ style "padding-top" "clamp(1.25rem, 4vw, 2rem)"
                    , style "margin-bottom" "1.5rem"
                    ]
                    [ UI.backButton (config.onNavigate (ProjectDetailRoute doc.projectId)) ]
                , documentHeader doc config
                , UI.card []
                    [ UI.cardHeader "Content"
                    , Components.Markdown.view doc.content
                    ]
                , div [ style "margin-top" "1.5rem" ]
                    [ UI.card []
                        [ UI.cardHeader "Info"
                        , div
                            [ style "display" "grid"
                            , style "grid-template-columns" "repeat(auto-fit, minmax(160px, 1fr))"
                            , style "gap" "1rem"
                            ]
                            [ metaItem "Type" (UI.docTypeBadge doc.documentType)
                            , metaItem "Version" (span [ style "font-family" UI.fontMono, style "color" UI.colors.textSecondary ] [ text (String.fromInt doc.version) ])
                            , metaItem "Created" (UI.timestamp doc.createdAt)
                            , metaItem "Updated" (UI.timestamp doc.updatedAt)
                            ]
                        ]
                    ]
                , div [ style "margin-top" "1.5rem" ]
                    [ case comments of
                        Success cmts ->
                            Components.Comments.viewThreaded
                                { sectionTitle = "Document Comments"
                                , emptyMessage = "No comments yet."
                                , formPlaceholder = "Add a comment on this document..."
                                , postLabel = "Post Comment"
                                , comments = cmts
                                , commentForm = commentForm
                                , isBusy = isBusy
                                , replyingToCommentId = replyingToCommentId
                                , editingCommentId = editingCommentId
                                , collapsedComments = collapsedComments
                                , voiceState = voiceState
                                , onCommentChange = config.onCommentChange
                                , onSubmitComment = config.onSubmitComment
                                , onSubmitReply = config.onSubmitReply
                                , onStartReply = config.onStartReply
                                , onCancelReply = config.onCancelReply
                                , onStartEdit = config.onStartEdit
                                , onSaveEdit = config.onSaveEdit
                                , onCancelEdit = config.onCancelEdit
                                , onToggleCollapse = config.onToggleCommentCollapse
                                , onVoiceStartRecording = config.onVoiceStartRecording
                                , onVoiceStopRecording = config.onVoiceStopRecording
                                , onVoiceReset = config.onVoiceReset
                                , onVoiceApplyToComment = config.onVoiceApplyToComment
                                , onVoiceEditComment = config.onVoiceEditComment
                                }

                        Loading ->
                            UI.loadingSpinner

                        _ ->
                            text ""
                    ]
                ]


documentHeader :
    WorkDocument
    -> { a | onRefresh : msg }
    -> Html msg
documentHeader doc config =
    div
        [ style "display" "flex"
        , style "justify-content" "space-between"
        , style "align-items" "flex-start"
        , style "flex-wrap" "wrap"
        , style "gap" "1rem"
        , style "margin-bottom" "1.5rem"
        , style "padding-bottom" "1rem"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        ]
        [ div []
            [ div
                [ style "display" "flex"
                , style "align-items" "baseline"
                , style "gap" "0.75rem"
                , style "margin-bottom" "0.5rem"
                ]
                [ h2
                    [ style "font-family" UI.fontDisplay
                    , style "font-size" "clamp(1.125rem, 4vw, 1.5rem)"
                    , style "font-weight" "600"
                    , style "color" UI.colors.textPrimary
                    , style "margin" "0"
                    ]
                    [ text doc.title ]
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.6875rem"
                    , style "color" UI.colors.textMuted
                    ]
                    [ text ("#" ++ String.fromInt doc.id) ]
                ]
            , div
                [ style "display" "flex"
                , style "gap" "0.5rem"
                , style "align-items" "center"
                ]
                [ UI.docTypeBadge doc.documentType
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.625rem"
                    , style "color" UI.colors.textMuted
                    ]
                    [ text ("v" ++ String.fromInt doc.version) ]
                ]
            ]
        , UI.iconButton "R" [ onClick config.onRefresh, title "Refresh" ]
        ]


metaItem : String -> Html msg -> Html msg
metaItem label content =
    div []
        [ div
            [ style "font-family" UI.fontMono
            , style "font-size" "0.5625rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.1em"
            , style "text-transform" "uppercase"
            , style "color" UI.colors.textMuted
            , style "margin-bottom" "0.25rem"
            ]
            [ text label ]
        , content
        ]
