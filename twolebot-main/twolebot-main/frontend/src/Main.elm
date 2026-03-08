port module Main exposing (main)

import Api
import Browser
import Browser.Dom
import Browser.Navigation as Nav
import Dict
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick)
import Http
import Json.Decode as D
import Pages.Capabilities
import Pages.Chat
import Pages.CronJobs
import Pages.Dashboard
import Pages.DocumentDetail
import Pages.LiveBoard
import Pages.Logs
import Pages.Messages
import Pages.ProjectDetail
import Pages.Projects
import Pages.Settings
import Pages.Setup
import Pages.TaskDetail
import Pages.Welcome
import Process
import Task
import Time
import Types exposing (..)
import UI
import Url
import Url.Parser as Parser exposing ((</>), (<?>))
import Url.Parser.Query as Query


-- ═══════════════════════════════════════════════════════════════════════════
-- PORTS
-- ═══════════════════════════════════════════════════════════════════════════


-- Outgoing: Elm → JS
port startRecording : () -> Cmd msg
port stopRecording : Bool -> Cmd msg
port subscribeChatWS : String -> Cmd msg
port unsubscribeChatWS : String -> Cmd msg
port scrollToBottom : String -> Cmd msg
port forceScrollToBottom : String -> Cmd msg
port triggerFileInput : () -> Cmd msg
port startVideoRecording : () -> Cmd msg
port stopVideoRecording : Bool -> Cmd msg
port logWarning : String -> Cmd msg

-- Incoming: JS → Elm
port audioRecorded : (D.Value -> msg) -> Sub msg
port audioRecordingError : (String -> msg) -> Sub msg
port chatWsMessage : (String -> msg) -> Sub msg
port fileSelected : (D.Value -> msg) -> Sub msg
port videoRecorded : (D.Value -> msg) -> Sub msg


main : Program () Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        , onUrlChange = UrlChanged
        , onUrlRequest = LinkClicked
        }


-- INIT

init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ url key =
    let
        route = urlToRoute url

        setupInitCmd =
            if isSetupRoute route then
                Cmd.none
            else
                Api.getSetupStatus GotInitSetupCheck
    in
    ( { navKey = key
      , route = route
      , backendOnline = False
      , feedData = NotAsked
      , responsesData = NotAsked
      , chats = NotAsked
      , messagesPage = NotAsked
      , messageSearch = ""
      , messagesPageIndex = 0
      , logsPage = NotAsked
      , logSearch = ""
      , logsPageIndex = 0
      , currentTime = Time.millisToPosix 0
      , settings = NotAsked
      , settingsSaving = False
      , apiKeys = Nothing
      , apiKeysLoading = False
      , apiKeysSaving = False
      , apiKeysError = Nothing
      , telegramTokenEdit = ""
      , geminiKeyEdit = ""
      , allowedUsernameInput = ""
      , cronJobs = NotAsked
      , cronStatus = NotAsked
      , semanticStatus = NotAsked
      , tunnelStatus = NotAsked
      , setupStatus = NotAsked
      , telegramTokenInput = ""
      , geminiKeyInput = ""
      , allowedUsernameSetupInput = ""
      -- Work state
      , workProjects = NotAsked
      , workProject = NotAsked
      , workTasks = NotAsked
      , workTask = NotAsked
      , workDocuments = NotAsked
      , workDocument = NotAsked
      , workComments = NotAsked
      , workActivity = NotAsked
      , workAnalytics = NotAsked
      , workLiveBoard = NotAsked
      -- Work UI state
      , projectTab = TasksTab
      , projectForm = emptyProjectForm
      , showProjectForm = False
      , taskForm = emptyTaskForm
      , showTaskForm = False
      , documentForm = emptyDocumentForm
      , showDocumentForm = False
      , commentForm = emptyCommentForm
      , replyingToCommentId = Nothing
      , editingCommentId = Nothing
      , collapsedComments = []
      , taskFilters = emptyTaskFilters
      , taskViewMode = BoardView
      , draggingTaskId = Nothing
      , boardDropComment = ""
      , boardDropTarget = Nothing
      , editingField = NotEditing
      , rejectReviewComment = ""
      , pendingStatusChange = Nothing
      , readyForReviewComment = ""
      , workBusy = False
      , workError = Nothing
      , workNotice = Nothing
      , voiceState = emptyVoiceState
      , chatState =
            case route of
                ChatRoute maybeConvId ->
                    { emptyChatPageState | activeChatId = maybeConvId }

                _ ->
                    emptyChatPageState
      }
    , Cmd.batch
        [ loadRouteData route 0 "" 0 ""
        , setupInitCmd
        , Api.getStatus GotStatusCheck
        ]
    )


urlToRoute : Url.Url -> Route
urlToRoute url =
    Parser.parse routeParser url
        |> Maybe.withDefault DashboardRoute


routeParser : Parser.Parser (Route -> a) a
routeParser =
    Parser.oneOf
        [ Parser.map DashboardRoute Parser.top
        , Parser.map DashboardRoute (Parser.s "dashboard")
        , Parser.map (MessagesRoute Nothing Nothing) (Parser.s "messages")
        , Parser.map (\id topic -> MessagesRoute (Just id) topic) (Parser.s "messages" </> Parser.string <?> Query.string "topic")
        , Parser.map LogsRoute (Parser.s "logs")
        , Parser.map CronJobsRoute (Parser.s "jobs")
        , Parser.map SettingsRoute (Parser.s "settings")
        , Parser.map CapabilitiesRoute (Parser.s "capabilities")
        , Parser.map WelcomeRoute (Parser.s "welcome")
        , Parser.map SetupRoute (Parser.s "setup")
        -- Work routes (detail routes before bare routes so Parser.oneOf matches correctly)
        , Parser.map ProjectDetailRoute (Parser.s "projects" </> Parser.int)
        , Parser.map ProjectsRoute (Parser.s "projects")
        , Parser.map TaskDetailRoute (Parser.s "tasks" </> Parser.int)
        , Parser.map DocumentDetailRoute (Parser.s "documents" </> Parser.int)
        , Parser.map LiveBoardRoute (Parser.s "live-board")
        , Parser.map (\cid -> ChatRoute (Just cid)) (Parser.s "chat" </> Parser.string)
        , Parser.map (ChatRoute Nothing) (Parser.s "chat")
        ]


isSetupRoute : Route -> Bool
isSetupRoute route =
    case route of
        SetupRoute -> True
        _ -> False


routeToUrl : Route -> String
routeToUrl route =
    case route of
        DashboardRoute -> "/"
        MessagesRoute Nothing _ -> "/messages"
        MessagesRoute (Just chatId) maybeTopic ->
            "/messages/" ++ chatId ++ topicQueryParam maybeTopic
        LogsRoute -> "/logs"
        CronJobsRoute -> "/jobs"
        SettingsRoute -> "/settings"
        CapabilitiesRoute -> "/capabilities"
        WelcomeRoute -> "/welcome"
        SetupRoute -> "/setup"
        -- Work routes
        ProjectsRoute -> "/projects"
        ProjectDetailRoute id -> "/projects/" ++ String.fromInt id
        TaskDetailRoute id -> "/tasks/" ++ String.fromInt id
        DocumentDetailRoute id -> "/documents/" ++ String.fromInt id
        LiveBoardRoute -> "/live-board"
        ChatRoute Nothing -> "/chat"
        ChatRoute (Just cid) -> "/chat/" ++ cid


topicQueryParam : Maybe String -> String
topicQueryParam maybeTopic =
    case maybeTopic of
        Just topic ->
            "?topic=" ++ topic

        Nothing ->
            ""


loadRouteData : Route -> Int -> String -> Int -> String -> Cmd Msg
loadRouteData route messagesPageIndex messageSearch logsPageIndex logSearch =
    case route of
        DashboardRoute ->
            Cmd.batch
                [ Api.getFeed GotFeed
                , Api.getResponses GotResponses
                , Api.getSemanticStatus GotSemanticStatus
                , Api.getTunnelStatus GotTunnelStatus
                ]

        MessagesRoute Nothing _ ->
            Api.getChats GotChats

        MessagesRoute (Just chatId) topicFilter ->
            Api.getMessages chatId messagesPageIndex 50 (if String.isEmpty messageSearch then Nothing else Just messageSearch) topicFilter GotMessagesPage

        LogsRoute ->
            Api.getLogs logsPageIndex 100 (if String.isEmpty logSearch then Nothing else Just logSearch) GotLogsPage

        CronJobsRoute ->
            Cmd.batch
                [ Api.getCronJobs GotCronJobs
                , Api.getCronStatus GotCronStatus
                ]

        SettingsRoute ->
            Cmd.batch
                [ Api.getSettings GotSettings
                , Api.getApiKeys GotApiKeys
                ]

        CapabilitiesRoute ->
            Cmd.none

        WelcomeRoute ->
            Cmd.none

        SetupRoute ->
            Api.getSetupStatus GotSetupStatus

        -- Work routes
        ProjectsRoute ->
            Api.listProjects GotProjects

        ProjectDetailRoute id ->
            Cmd.batch
                [ Api.getProject id GotProject
                , Api.listTasks (Just id) Nothing GotWorkTasks
                , Api.searchDocuments "" (Just id) GotWorkDocuments
                , Api.getRecentActivity 50 GotActivity
                , Api.getTaskAnalytics (Just id) GotAnalytics
                ]

        TaskDetailRoute id ->
            Cmd.batch
                [ Api.getTask id GotWorkTask
                , Api.listComments id GotComments
                ]

        DocumentDetailRoute id ->
            Cmd.batch
                [ Api.getDocument id GotDocument
                , Api.listCommentsForDocument id GotComments
                ]

        LiveBoardRoute ->
            Cmd.batch
                [ Api.getLiveBoard GotLiveBoard
                , Api.listProjects GotProjects
                ]

        ChatRoute maybeConvId ->
            Cmd.batch
                ([ Api.getConversations GotConversations ]
                    ++ (case maybeConvId of
                            Just cid ->
                                [ Api.getChatMessages cid (GotChatMessages cid)
                                , subscribeChatWS cid
                                , Task.attempt (\_ -> NoOp) (Browser.Dom.focus "chat-input")
                                ]

                            Nothing ->
                                []
                       )
                )


-- UPDATE

type Msg
    = NoOp
    | ClearWorkError
    | ClearWorkNotice
    | LinkClicked Browser.UrlRequest
    | UrlChanged Url.Url
    | Navigate Route
    | Refresh
    | Tick Time.Posix
    | GotFeed (Result Http.Error FeedData)
    | GotResponses (Result Http.Error ResponseFeedData)
    | GotChats (Result Http.Error (List ChatSummary))
    | GotMessagesPage (Result Http.Error MessagesPage)
    | GotLogsPage (Result Http.Error LogsPage)
    | BackToChats
    | MessagesPageChange Int
    | MessageSearchChange String
    | MessageSearchSubmit
    | LogsPageChange Int
    | LogSearchChange String
    | LogSearchSubmit
    -- Settings messages
    | GotSettings (Result Http.Error Settings)
    | ToggleToolMessages Bool
    | ToggleThinkingMessages Bool
    | ToggleToolResults Bool
    | ChangeChatHarness String
    | ChangeClaudeModel String
    | SaveClaudeModel
    | ChangeDevRolePrompt String
    | SaveDevRolePrompt
    | ChangeHardenRolePrompt String
    | SaveHardenRolePrompt
    | ChangePmRolePrompt String
    | SavePmRolePrompt
    | GotSettingsSaved (Result Http.Error Settings)
    -- API Keys messages
    | GotApiKeys (Result Http.Error Api.ApiKeysResponse)
    | TelegramTokenEditChange String
    | GeminiKeyEditChange String
    | SaveApiKeys
    | GotApiKeysSaved (Result Http.Error Api.UpdateApiKeysResponse)
    -- Allowed user messages
    | AllowedUsernameInputChange String
    | SaveAllowedUsername
    | ClearAllowedUsername
    -- Cron job messages
    | GotCronJobs (Result Http.Error (List CronJob))
    | GotCronStatus (Result Http.Error CronStatus)
    | PauseCronJob String
    | ResumeCronJob String
    | CancelCronJob String
    | GotCronJobUpdated (Result Http.Error CronJob)
    -- Semantic indexer messages
    | GotSemanticStatus (Result Http.Error SemanticStatus)
    | ToggleSemanticIndexer Bool
    | GotSemanticToggled (Result Http.Error SemanticStatus)
    | TriggerSemanticReindex
    | GotSemanticReindexed (Result Http.Error ())
    -- Tunnel messages
    | GotTunnelStatus (Result Http.Error TunnelStatus)
    -- Setup messages
    | GotInitSetupCheck (Result Http.Error Api.SetupStatusResponse)
    | GotSetupStatus (Result Http.Error Api.SetupStatusResponse)
    | TelegramTokenInput String
    | GeminiKeyInput String
    | SubmitTelegramToken
    | SubmitGeminiKey
    | InstallClaude
    | GotTelegramResponse (Result Http.Error Api.TelegramSetupResponse)
    | GotGeminiResponse (Result Http.Error Api.GeminiSetupResponse)
    | GotClaudeInstallResponse (Result Http.Error Api.ClaudeInstallResponse)
    | AllowedUsernameSetupInput String
    | SubmitAllowedUsername
    | GotAllowedUsernameSetupSaved (Result Http.Error Settings)
    | SkipGeminiStep
    | CheckClaudeAuth
    | GotClaudeAuthCheck (Result Http.Error Api.ClaudeAuthCheckResponse)
    | UpdateClaude
    | GotClaudeUpdateResponse (Result Http.Error Api.ClaudeInstallResponse)
    | TestClaude
    | GotClaudeTestResponse (Result Http.Error Api.ClaudeTestResponse)
    | CheckThreading
    | GotThreadingCheck (Result Http.Error Api.ThreadingCheckResponse)
    -- Work messages
    | GotProjects (Result Http.Error (List WorkProject))
    | GotProject (Result Http.Error WorkProject)
    | GotWorkTasks (Result Http.Error (List WorkTask))
    | GotWorkTask (Result Http.Error WorkTask)
    | GotWorkDocuments (Result Http.Error (List WorkDocument))
    | GotDocument (Result Http.Error WorkDocument)
    | GotComments (Result Http.Error (List WorkComment))
    | GotActivity (Result Http.Error (List ActivityLog))
    | GotAnalytics (Result Http.Error TaskAnalytics)
    | GotLiveBoard (Result Http.Error LiveBoard)
    -- Project form
    | ToggleProjectForm
    | CloseProjectForm
    | ProjectNameChange String
    | ProjectDescChange String
    | ProjectTagsChange String
    | ProjectGitRemoteUrlChange String
    | SubmitProject
    | GotProjectCreated (Result Http.Error WorkProject)
    -- Task form
    | ToggleTaskForm
    | CloseTaskForm
    | TaskTitleChange String
    | TaskDescChange String
    | TaskPriorityChange String
    | SubmitTask
    | GotTaskCreated (Result Http.Error WorkTask)
    -- Document form
    | ToggleDocForm
    | CloseDocForm
    | DocTitleChange String
    | DocContentChange String
    | DocTypeChange String
    | SubmitDocument
    | GotDocumentCreated (Result Http.Error WorkDocument)
    -- Task status / priority
    | ChangeTaskStatus String
    | ChangeTaskPriority String
    | GotTaskUpdated (Result Http.Error WorkTask)
    | GotBoardTaskUpdated (Result Http.Error WorkTask)
    | GotLiveBoardTaskUpdated (Result Http.Error WorkTask)
    -- Comments
    | CommentContentChange String
    | SubmitComment
    | SubmitReply Int
    | StartReply Int
    | CancelReply
    | StartEditComment Int
    | SaveEditedComment Int
    | CancelEditComment
    | ToggleCommentCollapse Int
    | GotCommentCreated (Result Http.Error WorkComment)
    -- Inline editing
    | StartEditField EditingField
    | CancelEditField
    | SaveEditField
    -- Board view
    | ToggleTaskViewMode
    | DragStart Int
    | DragEnd
    | DragOver
    | DropOnStatus String
    | BoardDropCommentChange String
    | SubmitBoardDrop
    | CancelBoardDrop
    -- Filters
    | ToggleStatusFilter String
    | ClearFilters
    -- Project detail tabs
    | ChangeProjectTab ProjectTab
    -- Live board actions
    | SelectLiveBoardTask Int
    | DeselectLiveBoardTask Int
    | MoveLiveSelection Int String
    | GotSelectionMoved (Result Http.Error ())
    | TakeNextTaskAction
    | GotTakeNextTaskAction (Result Http.Error ())
    | TakeNextReviewTaskAction
    | GotTakeNextReviewTaskAction (Result Http.Error ())
    | MoveTaskToTop
    | MoveTaskToBottom
    | RejectReviewCommentChange String
    | RejectReviewAction
    | ReadyForReviewCommentChange String
    | SubmitReadyForReview
    | CancelPendingStatus
    | GotTaskSelected (Result Http.Error (List LiveBoardSelection))
    | GotTaskDeselected (Result Http.Error ())
    | ClearCompletedTasks
    | GotClearedCompleted (Result Http.Error Int)
    | EnsureAgentLoopAction
    | StopAgentLoopAction
    | GotAgentLoopAction (Result Http.Error String)
    -- Voice messages
    | VoiceStartRecording VoiceMode
    | VoiceStopRecording
    | VoiceGotAudio String String  -- base64 audio data, MIME type
    | VoiceRecordingFailed String
    | GotTranscription (Result Http.Error Api.TranscribeResult)
    | GotFormatted (Result Http.Error Api.FormatResult)
    | VoiceReset
    | VoiceSetExistingContent String
    | VoiceApplyToTaskForm
    | VoiceApplyToTaskEdit
    | VoiceApplyToComment
    | VoiceEditComment Int
    -- Chat messages
    | GotConversations (Result Http.Error (List Conversation))
    | GotChatMessages String (Result Http.Error (List ChatMessage))
    | ChatNewConversation
    | GotConversationCreated (Result Http.Error Api.CreateConversationResponse)
    | ChatSelectConversation String
    | ChatInputChange String
    | ChatSendMessage
    | GotChatMessageSent String (Result Http.Error Api.SendMessageResponse)
    | ChatWsReceived String
    | ChatStartRename String String
    | ChatRenameChange String
    | ChatSubmitRename
    | ChatCancelRename
    | GotChatRenamed (Result Http.Error ())
    | ChatConfirmDelete String
    | ChatCancelDelete
    | ChatDeleteConversation String
    | GotChatDeleted String (Result Http.Error ())
    | ChatStartVoice
    | ChatStopVoice
    | ChatCancelVoice
    | ChatAttachFile
    | ChatFileReceived D.Value
    | ChatStartVideo
    | ChatStopVideo
    | ChatCancelVideo
    | ChatVideoReceived D.Value
    | GotChatMediaUploaded String String (Result Http.Error Api.UploadMediaResponse)
    | ChatDismissError String
    | ChatDismissNotification Int
    | ChatRetryUpload String String
    | ChatPollTelegram String
    -- Status check
    | GotStatusCheck (Result Http.Error ())


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )

        ClearWorkError ->
            ( { model | workError = Nothing }, Cmd.none )

        ClearWorkNotice ->
            ( { model | workNotice = Nothing }, Cmd.none )

        LinkClicked urlRequest ->
            case urlRequest of
                Browser.Internal url ->
                    ( model, Nav.pushUrl model.navKey (Url.toString url) )

                Browser.External href ->
                    ( model, Nav.load href )

        UrlChanged url ->
            let
                route = urlToRoute url

                clearedModel =
                    case route of
                        MessagesRoute (Just _) _ ->
                            { model | messagesPage = Loading, messagesPageIndex = 0, messageSearch = "" }

                        MessagesRoute Nothing _ ->
                            { model | chats = Loading }

                        ProjectDetailRoute _ ->
                            { model | workProject = Loading, workTasks = Loading, workDocuments = Loading, workActivity = Loading, workAnalytics = Loading }

                        TaskDetailRoute _ ->
                            { model
                                | workTask = Loading
                                , workComments = Loading
                                , commentForm = emptyCommentForm
                                , replyingToCommentId = Nothing
                                , editingCommentId = Nothing
                                , editingField = NotEditing
                                , rejectReviewComment = ""
                              }

                        DocumentDetailRoute _ ->
                            { model
                                | workDocument = Loading
                                , workComments = Loading
                                , commentForm = emptyCommentForm
                                , replyingToCommentId = Nothing
                                , editingCommentId = Nothing
                              }

                        ChatRoute newConvId ->
                            let
                                cs = model.chatState
                                newCs = { cs | activeChatId = newConvId }
                            in
                            case newConvId of
                                Just cid ->
                                    if isTelegramConversation cid newCs then
                                        { model | chatState =
                                            updateConversationState cid
                                                (\conv -> { conv | activity = ChatObserving })
                                                newCs
                                        }
                                    else
                                        { model | chatState = newCs }

                                Nothing ->
                                    { model | chatState = newCs }

                        _ ->
                            let
                                cs = model.chatState
                            in
                            { model | chatState = { cs | activeChatId = Nothing } }

                oldActiveChatId =
                    model.chatState.activeChatId

                newRoute =
                    route

                wsCleanup =
                    case ( oldActiveChatId, newRoute ) of
                        ( Just oldId, ChatRoute (Just newId) ) ->
                            if oldId /= newId then
                                unsubscribeChatWS oldId
                            else
                                Cmd.none

                        ( Just oldId, ChatRoute Nothing ) ->
                            unsubscribeChatWS oldId

                        ( Just oldId, _ ) ->
                            unsubscribeChatWS oldId

                        _ ->
                            Cmd.none

                voiceCleanup =
                    stopRecordingIfActive model.voiceState
            in
            ( { clearedModel | route = route, voiceState = emptyVoiceState }
            , Cmd.batch
                [ wsCleanup
                , voiceCleanup
                , loadRouteData route clearedModel.messagesPageIndex clearedModel.messageSearch model.logsPageIndex model.logSearch
                ]
            )

        Navigate route ->
            ( model, Nav.pushUrl model.navKey (routeToUrl route) )

        Refresh ->
            ( model
            , loadRouteData model.route model.messagesPageIndex model.messageSearch model.logsPageIndex model.logSearch
            )

        Tick time ->
            let
                statusCheck =
                    Api.getStatus GotStatusCheck
            in
            -- Don't auto-refresh on setup page
            if isSetupRoute model.route then
                ( { model | currentTime = time }, statusCheck )
            else
                let
                    shouldRefresh =
                        case model.route of
                            DashboardRoute ->
                                True

                            LogsRoute ->
                                model.logsPageIndex == 0 && String.isEmpty model.logSearch && (not (isLoading model.logsPage))

                            MessagesRoute Nothing _ ->
                                not (isLoading model.chats)

                            MessagesRoute (Just _) _ ->
                                model.messagesPageIndex == 0 && String.isEmpty model.messageSearch && (not (isLoading model.messagesPage))

                            CronJobsRoute ->
                                True

                            SettingsRoute ->
                                False

                            CapabilitiesRoute ->
                                False

                            WelcomeRoute ->
                                False

                            SetupRoute ->
                                False

                            LiveBoardRoute ->
                                True

                            ProjectsRoute ->
                                False

                            ProjectDetailRoute _ ->
                                False

                            TaskDetailRoute _ ->
                                False

                            DocumentDetailRoute _ ->
                                False

                            ChatRoute _ ->
                                False
                in
                if shouldRefresh then
                    ( { model | currentTime = time }
                    , Cmd.batch
                        [ loadRouteData model.route model.messagesPageIndex model.messageSearch model.logsPageIndex model.logSearch
                        , statusCheck
                        ]
                    )
                else
                    ( { model | currentTime = time }, statusCheck )

        GotStatusCheck result ->
            ( { model | backendOnline = result |> Result.map (\_ -> True) |> Result.withDefault False }, Cmd.none )

        GotFeed result ->
            ( { model | feedData = resultToRemote result }, Cmd.none )

        GotResponses result ->
            ( { model | responsesData = resultToRemote result }, Cmd.none )

        GotChats result ->
            ( { model | chats = resultToRemote result }, Cmd.none )

        GotMessagesPage result ->
            case result of
                Ok page ->
                    ( { model | messagesPage = Success page, messagesPageIndex = page.page }, Cmd.none )

                Err err ->
                    ( { model | messagesPage = Failure (httpErrorToString err) }, Cmd.none )

        GotLogsPage result ->
            case result of
                Ok page ->
                    ( { model | logsPage = Success page, logsPageIndex = page.page }, Cmd.none )

                Err err ->
                    ( { model | logsPage = Failure (httpErrorToString err) }, Cmd.none )

        BackToChats ->
            ( { model | messagesPage = NotAsked, messageSearch = "", messagesPageIndex = 0 }
            , Nav.pushUrl model.navKey (routeToUrl (MessagesRoute Nothing Nothing))
            )

        MessagesPageChange page ->
            case model.route of
                MessagesRoute (Just chatId) topicFilter ->
                    let
                        search = if String.isEmpty model.messageSearch then Nothing else Just model.messageSearch
                    in
                    ( { model | messagesPage = Loading, messagesPageIndex = page }
                    , Api.getMessages chatId page 50 search topicFilter GotMessagesPage
                    )
                _ ->
                    ( model, Cmd.none )

        MessageSearchChange search ->
            ( { model | messageSearch = search }, Cmd.none )

        MessageSearchSubmit ->
            case model.route of
                MessagesRoute (Just chatId) topicFilter ->
                    let
                        search = if String.isEmpty model.messageSearch then Nothing else Just model.messageSearch
                    in
                    ( { model | messagesPage = Loading, messagesPageIndex = 0 }
                    , Api.getMessages chatId 0 50 search topicFilter GotMessagesPage
                    )
                _ ->
                    ( model, Cmd.none )

        LogsPageChange page ->
            ( { model | logsPage = Loading, logsPageIndex = page }
            , Api.getLogs page 100 (if String.isEmpty model.logSearch then Nothing else Just model.logSearch) GotLogsPage
            )

        LogSearchChange search ->
            ( { model | logSearch = search }, Cmd.none )

        LogSearchSubmit ->
            ( { model | logsPage = Loading, logsPageIndex = 0 }
            , Api.getLogs 0 100 (if String.isEmpty model.logSearch then Nothing else Just model.logSearch) GotLogsPage
            )

        -- Settings handlers
        GotSettings result ->
            ( { model | settings = resultToRemote result }, Cmd.none )

        ToggleToolMessages value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | showToolMessages = value }
                    in
                    ( { model | settings = Success newSettings, settingsSaving = True }
                    , Api.putSettings newSettings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ToggleThinkingMessages value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | showThinkingMessages = value }
                    in
                    ( { model | settings = Success newSettings, settingsSaving = True }
                    , Api.putSettings newSettings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ToggleToolResults value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | showToolResults = value }
                    in
                    ( { model | settings = Success newSettings, settingsSaving = True }
                    , Api.putSettings newSettings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ChangeChatHarness value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | chatHarness = normalizeChatHarness value }
                    in
                    ( { model | settings = Success newSettings, settingsSaving = True }
                    , Api.putSettings newSettings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ChangeClaudeModel value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | claudeModel = value }
                    in
                    ( { model | settings = Success newSettings }, Cmd.none )
                _ ->
                    ( model, Cmd.none )

        SaveClaudeModel ->
            case model.settings of
                Success settings ->
                    let
                        trimmed = String.trim settings.claudeModel
                        newSettings = { settings | claudeModel = trimmed }
                    in
                    ( { model | settings = Success newSettings, settingsSaving = True }
                    , Api.putSettings newSettings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ChangeDevRolePrompt value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | devRolePrompt = value }
                    in
                    ( { model | settings = Success newSettings }, Cmd.none )
                _ ->
                    ( model, Cmd.none )

        SaveDevRolePrompt ->
            case model.settings of
                Success settings ->
                    ( { model | settingsSaving = True }
                    , Api.putSettings settings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ChangeHardenRolePrompt value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | hardenRolePrompt = value }
                    in
                    ( { model | settings = Success newSettings }, Cmd.none )
                _ ->
                    ( model, Cmd.none )

        SaveHardenRolePrompt ->
            case model.settings of
                Success settings ->
                    ( { model | settingsSaving = True }
                    , Api.putSettings settings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        ChangePmRolePrompt value ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | pmRolePrompt = value }
                    in
                    ( { model | settings = Success newSettings }, Cmd.none )
                _ ->
                    ( model, Cmd.none )

        SavePmRolePrompt ->
            case model.settings of
                Success settings ->
                    ( { model | settingsSaving = True }
                    , Api.putSettings settings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        GotSettingsSaved result ->
            case result of
                Ok settings ->
                    ( { model | settings = Success settings, settingsSaving = False }, Cmd.none )
                Err _ ->
                    -- Keep the optimistic update, just clear saving state
                    ( { model | settingsSaving = False }, Cmd.none )

        AllowedUsernameInputChange value ->
            ( { model | allowedUsernameInput = value }, Cmd.none )

        SaveAllowedUsername ->
            case model.settings of
                Success settings ->
                    let
                        username = sanitizeUsername model.allowedUsernameInput
                    in
                    if String.isEmpty username then
                        ( model, Cmd.none )
                    else
                        let
                            newSettings = { settings | allowedUsername = Just username }
                        in
                        ( { model | settings = Success newSettings, settingsSaving = True, allowedUsernameInput = "" }
                        , Api.putSettings newSettings GotSettingsSaved
                        )
                _ ->
                    ( model, Cmd.none )

        ClearAllowedUsername ->
            case model.settings of
                Success settings ->
                    let
                        newSettings = { settings | allowedUsername = Nothing }
                    in
                    ( { model | settings = Success newSettings, settingsSaving = True }
                    , Api.putSettings newSettings GotSettingsSaved
                    )
                _ ->
                    ( model, Cmd.none )

        -- API Keys handlers
        GotApiKeys result ->
            case result of
                Ok data ->
                    ( { model
                        | apiKeys = Just (apiKeysResponseToData data)
                        , apiKeysLoading = False
                        , apiKeysError = Nothing
                      }
                    , Cmd.none
                    )
                Err err ->
                    ( { model | apiKeysLoading = False, apiKeysError = Just (httpErrorToString err) }, Cmd.none )

        TelegramTokenEditChange token ->
            ( { model | telegramTokenEdit = token }, Cmd.none )

        GeminiKeyEditChange key ->
            ( { model | geminiKeyEdit = key }, Cmd.none )

        SaveApiKeys ->
            let
                telegramToken = if String.isEmpty model.telegramTokenEdit then Nothing else Just model.telegramTokenEdit
                geminiKey = if String.isEmpty model.geminiKeyEdit then Nothing else Just model.geminiKeyEdit
            in
            if telegramToken == Nothing && geminiKey == Nothing then
                ( model, Cmd.none )
            else
                ( { model | apiKeysSaving = True, apiKeysError = Nothing }
                , Api.putApiKeys telegramToken geminiKey GotApiKeysSaved
                )

        GotApiKeysSaved result ->
            case result of
                Ok response ->
                    let
                        error =
                            case ( response.telegramError, response.geminiError ) of
                                ( Just te, Just ge ) -> Just (te ++ "; " ++ ge)
                                ( Just te, Nothing ) -> Just te
                                ( Nothing, Just ge ) -> Just ge
                                ( Nothing, Nothing ) -> Nothing
                    in
                    ( { model
                        | apiKeysSaving = False
                        , apiKeysError = error
                        , telegramTokenEdit = if response.telegramUpdated then "" else model.telegramTokenEdit
                        , geminiKeyEdit = if response.geminiUpdated then "" else model.geminiKeyEdit
                      }
                    , if response.telegramUpdated || response.geminiUpdated then
                        Api.getApiKeys GotApiKeys
                      else
                        Cmd.none
                    )
                Err err ->
                    ( { model | apiKeysSaving = False, apiKeysError = Just (httpErrorToString err) }, Cmd.none )

        -- Cron job handlers
        GotCronJobs result ->
            ( { model | cronJobs = resultToRemote result }, Cmd.none )

        GotCronStatus result ->
            ( { model | cronStatus = resultToRemote result }, Cmd.none )

        PauseCronJob jobId ->
            ( model, Api.pauseCronJob jobId GotCronJobUpdated )

        ResumeCronJob jobId ->
            ( model, Api.resumeCronJob jobId GotCronJobUpdated )

        CancelCronJob jobId ->
            ( model, Api.cancelCronJob jobId GotCronJobUpdated )

        GotCronJobUpdated result ->
            case result of
                Ok updatedJob ->
                    -- Update the job in the list and refresh status
                    let
                        updateJob job =
                            if job.id == updatedJob.id then
                                updatedJob
                            else
                                job

                        updatedJobs =
                            case model.cronJobs of
                                Success jobs ->
                                    Success (List.map updateJob jobs)

                                other ->
                                    other
                    in
                    ( { model | cronJobs = updatedJobs }
                    , Api.getCronStatus GotCronStatus
                    )

                Err _ ->
                    -- On error, just refresh everything
                    ( model
                    , Cmd.batch
                        [ Api.getCronJobs GotCronJobs
                        , Api.getCronStatus GotCronStatus
                        ]
                    )

        -- Semantic indexer handlers
        GotSemanticStatus result ->
            ( { model | semanticStatus = resultToRemote result }, Cmd.none )

        ToggleSemanticIndexer enabled ->
            -- Optimistic update
            case model.semanticStatus of
                Success status ->
                    ( { model | semanticStatus = Success { status | enabled = enabled } }
                    , Api.toggleSemantic enabled GotSemanticToggled
                    )
                _ ->
                    ( model, Api.toggleSemantic enabled GotSemanticToggled )

        GotSemanticToggled result ->
            case result of
                Ok status ->
                    ( { model | semanticStatus = Success status }, Cmd.none )
                Err _ ->
                    -- Revert optimistic update by re-fetching
                    ( model, Api.getSemanticStatus GotSemanticStatus )

        TriggerSemanticReindex ->
            ( model, Api.triggerSemanticReindex GotSemanticReindexed )

        GotSemanticReindexed _ ->
            -- Refresh status regardless of success/failure
            ( model, Api.getSemanticStatus GotSemanticStatus )

        GotTunnelStatus result ->
            ( { model | tunnelStatus = resultToRemote result }, Cmd.none )

        -- Setup handlers
        GotInitSetupCheck result ->
            case result of
                Ok resp ->
                    ( { model | setupStatus = Success (responseToSetupStatus resp) }, Cmd.none )

                Err err ->
                    ( { model | setupStatus = Failure (httpErrorToString err) }, Cmd.none )

        GotSetupStatus result ->
            case result of
                Ok resp ->
                    let
                        status = responseToSetupStatus resp
                        authCmd =
                            if status.hasClaudeCli then
                                Api.getClaudeAuth GotClaudeAuthCheck
                            else
                                Cmd.none
                    in
                    ( { model | setupStatus = Success status }, authCmd )

                Err err ->
                    ( { model | setupStatus = Failure (httpErrorToString err) }, Cmd.none )

        TelegramTokenInput token ->
            ( { model | telegramTokenInput = token }, Cmd.none )

        GeminiKeyInput key ->
            ( { model | geminiKeyInput = key }, Cmd.none )

        SubmitTelegramToken ->
            ( model, Api.postTelegramToken model.telegramTokenInput GotTelegramResponse )

        SubmitGeminiKey ->
            ( model, Api.postGeminiKey model.geminiKeyInput GotGeminiResponse )

        InstallClaude ->
            let
                currentStatus = currentSetupStatus model
                newStatus = { currentStatus | claudeInstalling = True, claudeInstallError = Nothing }
            in
            ( { model | setupStatus = Success newStatus }, Api.postInstallClaude GotClaudeInstallResponse )

        GotTelegramResponse result ->
            case result of
                Ok resp ->
                    let
                        currentStatus = currentSetupStatus model
                        newStatus =
                            if resp.success then
                                { currentStatus
                                    | hasTelegramToken = True
                                    , botName = resp.botName
                                    , telegramError = Nothing
                                }
                            else
                                { currentStatus | telegramError = resp.error }
                    in
                    ( { model | setupStatus = Success newStatus, telegramTokenInput = "" }, Cmd.none )

                Err err ->
                    let
                        currentStatus = currentSetupStatus model
                        newStatus = { currentStatus | telegramError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        GotGeminiResponse result ->
            case result of
                Ok resp ->
                    let
                        currentStatus = currentSetupStatus model
                        key = model.geminiKeyInput
                        keyPreview =
                            if String.length key >= 8 then
                                Just (String.left 4 key ++ "..." ++ String.right 4 key)
                            else if not (String.isEmpty key) then
                                Just key
                            else
                                Nothing
                        newStatus =
                            if resp.success then
                                { currentStatus
                                    | hasGeminiKey = True
                                    , geminiError = Nothing
                                    , geminiKeyPreview = keyPreview
                                }
                            else
                                { currentStatus | geminiError = resp.error }
                    in
                    ( { model | setupStatus = Success newStatus, geminiKeyInput = "" }, Cmd.none )

                Err err ->
                    let
                        currentStatus = currentSetupStatus model
                        newStatus = { currentStatus | geminiError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        GotClaudeInstallResponse result ->
            case result of
                Ok resp ->
                    let
                        currentStatus = currentSetupStatus model
                        newStatus =
                            if resp.success then
                                { currentStatus
                                    | hasClaudeCli = True
                                    , claudeCliVersion = resp.version
                                    , claudeInstalling = False
                                    , claudeInstallError = Nothing
                                }
                            else
                                { currentStatus
                                    | claudeInstalling = False
                                    , claudeInstallError = resp.error
                                }
                    in
                    -- After install, auto-check auth status
                    ( { model | setupStatus = Success newStatus }
                    , if resp.success then Api.getClaudeAuth GotClaudeAuthCheck else Cmd.none
                    )

                Err err ->
                    let
                        currentStatus = currentSetupStatus model
                        newStatus = { currentStatus | claudeInstalling = False, claudeInstallError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        AllowedUsernameSetupInput value ->
            ( { model | allowedUsernameSetupInput = value }, Cmd.none )

        SubmitAllowedUsername ->
            let
                username = sanitizeUsername model.allowedUsernameSetupInput
                baseSettings =
                    case model.settings of
                        Success settings -> settings
                        _ -> defaultSettings
            in
            if String.isEmpty username then
                ( model, Cmd.none )
            else
                let
                    newSettings = { baseSettings | allowedUsername = Just username }
                in
                ( { model | settings = Success newSettings, settingsSaving = True }
                , Api.putSettings newSettings GotAllowedUsernameSetupSaved
                )

        GotAllowedUsernameSetupSaved result ->
            case result of
                Ok settings ->
                    let
                        currentStatus = currentSetupStatus model
                        savedName = settings.allowedUsername
                        newStatus = { currentStatus | hasAllowedUsername = True, allowedUsernameError = Nothing, allowedUsernameValue = savedName }
                    in
                    ( { model
                        | settings = Success settings
                        , settingsSaving = False
                        , setupStatus = Success newStatus
                        , allowedUsernameSetupInput = ""
                      }
                    , Cmd.none
                    )

                Err err ->
                    let
                        currentStatus = currentSetupStatus model
                        newStatus = { currentStatus | allowedUsernameError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus, settingsSaving = False }, Cmd.none )

        SkipGeminiStep ->
            ( model, Cmd.none )

        CheckClaudeAuth ->
            let
                currentStatus = currentSetupStatus model
                newStatus = { currentStatus | claudeAuthChecking = True }
            in
            ( { model | setupStatus = Success newStatus }, Api.getClaudeAuth GotClaudeAuthCheck )

        GotClaudeAuthCheck result ->
            let
                currentStatus = currentSetupStatus model
            in
            case result of
                Ok resp ->
                    let
                        newStatus =
                            { currentStatus
                                | hasClaudeCli = resp.installed
                                , claudeCliVersion = resp.version
                                , claudeAuthenticated = resp.authenticated
                                , claudeAuthMode = resp.authMode
                                , claudeAccountEmail = resp.accountEmail
                                , claudeAccountName = resp.accountName
                                , claudeNeedsUpdate = resp.needsUpdate
                                , claudeLatestVersion = resp.latestVersion
                                , claudeAuthChecking = False
                            }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

                Err err ->
                    let
                        newStatus = { currentStatus | claudeAuthChecking = False, claudeInstallError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        UpdateClaude ->
            let
                currentStatus = currentSetupStatus model
                newStatus = { currentStatus | claudeUpdating = True, claudeUpdateError = Nothing }
            in
            ( { model | setupStatus = Success newStatus }, Api.postUpdateClaude GotClaudeUpdateResponse )

        GotClaudeUpdateResponse result ->
            let
                currentStatus = currentSetupStatus model
            in
            case result of
                Ok resp ->
                    let
                        newStatus =
                            if resp.success then
                                { currentStatus
                                    | claudeCliVersion = resp.version
                                    , claudeNeedsUpdate = False
                                    , claudeUpdating = False
                                    , claudeUpdateError = Nothing
                                }
                            else
                                { currentStatus
                                    | claudeUpdating = False
                                    , claudeUpdateError = resp.error
                                }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

                Err err ->
                    let
                        newStatus = { currentStatus | claudeUpdating = False, claudeUpdateError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        TestClaude ->
            let
                currentStatus = currentSetupStatus model
                newStatus = { currentStatus | claudeTesting = True, claudeTestResult = Nothing, claudeTestOutput = Nothing, claudeTestError = Nothing }
            in
            ( { model | setupStatus = Success newStatus }, Api.postTestClaude GotClaudeTestResponse )

        GotClaudeTestResponse result ->
            let
                currentStatus = currentSetupStatus model
            in
            case result of
                Ok resp ->
                    let
                        newStatus =
                            { currentStatus
                                | claudeTesting = False
                                , claudeTestResult = Just resp.success
                                , claudeTestOutput = resp.output
                                , claudeTestError = resp.error
                            }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

                Err err ->
                    let
                        newStatus = { currentStatus | claudeTesting = False, claudeTestResult = Just False, claudeTestError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        CheckThreading ->
            let
                currentStatus = currentSetupStatus model
                newStatus = { currentStatus | threadingChecking = True, threadingError = Nothing }
            in
            ( { model | setupStatus = Success newStatus }, Api.checkThreading GotThreadingCheck )

        GotThreadingCheck result ->
            let
                currentStatus = currentSetupStatus model
            in
            case result of
                Ok resp ->
                    let
                        newStatus =
                            { currentStatus
                                | threadingChecking = False
                                , hasThreadingEnabled = resp.enabled
                                , threadingError =
                                    if resp.enabled then
                                        Nothing
                                    else
                                        resp.error
                                            |> Maybe.withDefault "Threading not enabled yet. Follow the steps above, then check again."
                                            |> Just
                            }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

                Err err ->
                    let
                        newStatus = { currentStatus | threadingChecking = False, threadingError = Just (httpErrorToString err) }
                    in
                    ( { model | setupStatus = Success newStatus }, Cmd.none )

        -- ═══════════════════════════════════════════════════════════════
        -- WORK HANDLERS
        -- ═══════════════════════════════════════════════════════════════

        GotProjects result ->
            ( { model | workProjects = resultToRemote result }, Cmd.none )

        GotProject result ->
            ( { model | workProject = resultToRemote result }, Cmd.none )

        GotWorkTasks result ->
            ( { model | workTasks = resultToRemote result }, Cmd.none )

        GotWorkTask result ->
            ( { model | workTask = resultToRemote result }, Cmd.none )

        GotWorkDocuments result ->
            ( { model | workDocuments = resultToRemote result }, Cmd.none )

        GotDocument result ->
            ( { model | workDocument = resultToRemote result }, Cmd.none )

        GotComments result ->
            ( { model | workComments = resultToRemote result }, Cmd.none )

        GotActivity result ->
            ( { model | workActivity = resultToRemote result }, Cmd.none )

        GotAnalytics result ->
            ( { model | workAnalytics = resultToRemote result }, Cmd.none )

        GotLiveBoard result ->
            case result of
                Ok board ->
                    let
                        maybePrevStats =
                            case model.workLiveBoard of
                                Success prev ->
                                    Just prev.stats

                                _ ->
                                    Nothing

                        maybeNotice =
                            case maybePrevStats of
                                Just prev ->
                                    if board.stats.completed > prev.completed then
                                        Just ("Agent completed " ++ String.fromInt (board.stats.completed - prev.completed) ++ " task(s).")
                                    else if board.stats.failed > prev.failed then
                                        Just ("Agent failed " ++ String.fromInt (board.stats.failed - prev.failed) ++ " task(s). Check task comments for details.")
                                    else
                                        Nothing

                                Nothing ->
                                    Nothing

                        shouldEnsureAgent =
                            model.route == LiveBoardRoute
                                && board.stats.agentLoopState == "idle"
                                && (board.stats.queued > 0 || board.stats.totalBacklog > 0)
                                && (not model.workBusy)

                        noticeCmd =
                            case maybeNotice of
                                Just _ ->
                                    Process.sleep 5000 |> Task.perform (\_ -> ClearWorkNotice)

                                Nothing ->
                                    Cmd.none
                    in
                    ( { model
                        | workLiveBoard = Success board
                        , workNotice =
                            case maybeNotice of
                                Just notice ->
                                    Just notice

                                Nothing ->
                                    model.workNotice
                      }
                    , Cmd.batch
                        [ noticeCmd
                        , if shouldEnsureAgent then
                            Api.ensureAgentLoop GotAgentLoopAction
                          else
                            Cmd.none
                        ]
                    )

                Err err ->
                    ( { model | workLiveBoard = Failure (httpErrorToString err) }, Cmd.none )

        -- Project form
        ToggleProjectForm ->
            ( { model | showProjectForm = not model.showProjectForm, projectForm = emptyProjectForm }, Cmd.none )

        CloseProjectForm ->
            ( { model | showProjectForm = False, projectForm = emptyProjectForm }, Cmd.none )

        ProjectNameChange val ->
            let form = model.projectForm in
            ( { model | projectForm = { form | name = val } }, Cmd.none )

        ProjectDescChange val ->
            let form = model.projectForm in
            ( { model | projectForm = { form | description = val } }, Cmd.none )

        ProjectTagsChange val ->
            let form = model.projectForm in
            ( { model | projectForm = { form | tags = val } }, Cmd.none )

        ProjectGitRemoteUrlChange val ->
            let form = model.projectForm in
            ( { model | projectForm = { form | gitRemoteUrl = val } }, Cmd.none )

        SubmitProject ->
            let
                form = model.projectForm
                tags =
                    form.tags
                        |> String.split ","
                        |> List.map String.trim
                        |> List.filter (not << String.isEmpty)
                gitRemoteUrl =
                    if String.isEmpty (String.trim form.gitRemoteUrl) then
                        Nothing
                    else
                        Just (String.trim form.gitRemoteUrl)
            in
            ( { model | workBusy = True }
            , Api.createProject form.name form.description tags gitRemoteUrl GotProjectCreated
            )

        GotProjectCreated result ->
            case result of
                Ok _ ->
                    ( { model | workError = Nothing, workBusy = False, showProjectForm = False, projectForm = emptyProjectForm }, Api.listProjects GotProjects )
                Err err ->
                    setWorkError ("Failed to create project: " ++ httpErrorToString err) model

        -- Task form
        ToggleTaskForm ->
            let
                vs = emptyVoiceState
            in
            ( { model | showTaskForm = not model.showTaskForm, taskForm = emptyTaskForm, voiceState = { vs | mode = VoiceTicket } }
            , stopRecordingIfActive model.voiceState
            )

        CloseTaskForm ->
            ( { model | showTaskForm = False, taskForm = emptyTaskForm, voiceState = emptyVoiceState }
            , stopRecordingIfActive model.voiceState
            )

        TaskTitleChange val ->
            let form = model.taskForm in
            ( { model | taskForm = { form | title = val } }, Cmd.none )

        TaskDescChange val ->
            let form = model.taskForm in
            ( { model | taskForm = { form | description = val } }, Cmd.none )

        TaskPriorityChange val ->
            let form = model.taskForm in
            ( { model | taskForm = { form | priority = val } }, Cmd.none )

        SubmitTask ->
            case model.workProject of
                Success proj ->
                    let form = model.taskForm in
                    ( { model | workBusy = True }
                    , Api.createTask proj.id form.title form.description form.priority [] GotTaskCreated
                    )
                _ ->
                    ( model, Cmd.none )

        GotTaskCreated result ->
            case result of
                Ok _ ->
                    case model.workProject of
                        Success proj ->
                            ( { model | workError = Nothing, workBusy = False, showTaskForm = False, taskForm = emptyTaskForm }
                            , Cmd.batch
                                [ Api.listTasks (Just proj.id) Nothing GotWorkTasks
                                , Api.getTaskAnalytics (Just proj.id) GotAnalytics
                                ]
                            )
                        _ ->
                            ( { model | workBusy = False, showTaskForm = False, taskForm = emptyTaskForm }, Cmd.none )
                Err err ->
                    setWorkError ("Failed to create task: " ++ httpErrorToString err) model

        -- Document form
        ToggleDocForm ->
            ( { model | showDocumentForm = not model.showDocumentForm, documentForm = emptyDocumentForm }, Cmd.none )

        CloseDocForm ->
            ( { model | showDocumentForm = False, documentForm = emptyDocumentForm }, Cmd.none )

        DocTitleChange val ->
            let form = model.documentForm in
            ( { model | documentForm = { form | title = val } }, Cmd.none )

        DocContentChange val ->
            let form = model.documentForm in
            ( { model | documentForm = { form | content = val } }, Cmd.none )

        DocTypeChange val ->
            let form = model.documentForm in
            ( { model | documentForm = { form | documentType = val } }, Cmd.none )

        SubmitDocument ->
            case model.workProject of
                Success proj ->
                    let form = model.documentForm in
                    ( { model | workBusy = True }
                    , Api.createDocument proj.id form.title form.content form.documentType GotDocumentCreated
                    )
                _ ->
                    ( model, Cmd.none )

        GotDocumentCreated result ->
            case result of
                Ok _ ->
                    case model.workProject of
                        Success proj ->
                            ( { model | workError = Nothing, workBusy = False, showDocumentForm = False, documentForm = emptyDocumentForm }, Api.searchDocuments "" (Just proj.id) GotWorkDocuments )
                        _ ->
                            ( { model | workBusy = False, showDocumentForm = False, documentForm = emptyDocumentForm }, Cmd.none )
                Err err ->
                    setWorkError ("Failed to create document: " ++ httpErrorToString err) model

        -- Task status change
        ChangeTaskStatus newStatus ->
            if newStatus == "ready_for_review" then
                ( { model | pendingStatusChange = Just "ready_for_review", readyForReviewComment = "" }, Cmd.none )
            else
                case model.workTask of
                    Success task ->
                        ( { model | workBusy = True }
                        , Api.updateTask task.id
                            { title = Nothing, description = Nothing, status = Just newStatus, priority = Nothing, tags = Nothing, comment = Nothing }
                            GotTaskUpdated
                        )
                    _ ->
                        ( model, Cmd.none )

        ChangeTaskPriority newPriority ->
            case model.workTask of
                Success task ->
                    ( { model | workBusy = True, editingField = NotEditing }
                    , Api.updateTask task.id
                        { title = Nothing, description = Nothing, status = Nothing, priority = Just newPriority, tags = Nothing, comment = Nothing }
                        GotTaskUpdated
                    )
                _ ->
                    ( { model | editingField = NotEditing }, Cmd.none )

        GotTaskUpdated result ->
            case result of
                Ok updatedTask ->
                    let
                        refreshCmd =
                            case model.route of
                                TaskDetailRoute _ ->
                                    Api.listComments updatedTask.id GotComments

                                _ ->
                                    Cmd.none
                    in
                    ( { model
                        | workTask = Success updatedTask
                        , workError = Nothing
                        , workBusy = False
                        , pendingStatusChange = Nothing
                        , readyForReviewComment = ""
                        , rejectReviewComment = ""
                      }
                    , refreshCmd
                    )
                Err err ->
                    setWorkError ("Failed to update task: " ++ httpErrorToString err) model

        GotBoardTaskUpdated result ->
            case result of
                Ok _ ->
                    workSuccess model (refreshCurrentProjectTasksAndAnalytics model)

                Err err ->
                    setWorkError ("Failed to update task: " ++ httpErrorToString err) model

        GotLiveBoardTaskUpdated result ->
            case result of
                Ok _ ->
                    workSuccess model (Api.getLiveBoard GotLiveBoard)

                Err err ->
                    setWorkError ("Failed to update task: " ++ httpErrorToString err) model

        -- Comments
        CommentContentChange val ->
            ( { model | commentForm = { content = val } }, Cmd.none )

        SubmitComment ->
            let
                content = String.trim model.commentForm.content
            in
            case commentCmd model { commentId = Nothing, content = content, parentCommentId = Nothing } of
                Just cmd ->
                    if String.isEmpty content then ( model, Cmd.none )
                    else ( { model | workBusy = True }, cmd )

                Nothing ->
                    ( model, Cmd.none )

        SubmitReply parentCommentId ->
            let
                content = String.trim model.commentForm.content
            in
            case commentCmd model { commentId = Nothing, content = content, parentCommentId = Just parentCommentId } of
                Just cmd ->
                    if String.isEmpty content then ( model, Cmd.none )
                    else ( { model | workBusy = True }, cmd )

                Nothing ->
                    ( model, Cmd.none )

        StartReply commentId ->
            ( { model
                | replyingToCommentId = Just commentId
                , editingCommentId = Nothing
                , commentForm = emptyCommentForm
              }
            , Cmd.none
            )

        CancelReply ->
            ( { model | replyingToCommentId = Nothing, commentForm = emptyCommentForm }, Cmd.none )

        StartEditComment commentId ->
            let
                existingContent =
                    case model.workComments of
                        Success comments ->
                            comments
                                |> List.filter (\c -> c.id == commentId)
                                |> List.head
                                |> Maybe.map .content
                                |> Maybe.withDefault ""

                        _ ->
                            ""
            in
            ( { model
                | editingCommentId = Just commentId
                , replyingToCommentId = Nothing
                , commentForm = { content = existingContent }
              }
            , Cmd.none
            )

        SaveEditedComment commentId ->
            let
                content = String.trim model.commentForm.content
            in
            case commentCmd model { commentId = Just commentId, content = content, parentCommentId = Nothing } of
                Just cmd ->
                    if String.isEmpty content then ( model, Cmd.none )
                    else ( { model | workBusy = True }, cmd )

                Nothing ->
                    ( model, Cmd.none )

        CancelEditComment ->
            ( { model | editingCommentId = Nothing, commentForm = emptyCommentForm, voiceState = emptyVoiceState }
            , stopRecordingIfActive model.voiceState
            )

        ToggleCommentCollapse commentId ->
            let
                collapsed =
                    if List.member commentId model.collapsedComments then
                        List.filter (\c -> c /= commentId) model.collapsedComments
                    else
                        commentId :: model.collapsedComments
            in
            ( { model | collapsedComments = collapsed }, Cmd.none )

        GotCommentCreated result ->
            case result of
                Ok _ ->
                    case model.route of
                        TaskDetailRoute _ ->
                            case model.workTask of
                                Success task ->
                                    ( { model
                                        | workError = Nothing
                                        , workBusy = False
                                        , commentForm = emptyCommentForm
                                        , replyingToCommentId = Nothing
                                        , editingCommentId = Nothing
                                      }
                                    , Api.listComments task.id GotComments
                                    )

                                _ ->
                                    ( { model | commentForm = emptyCommentForm }, Cmd.none )

                        DocumentDetailRoute _ ->
                            case model.workDocument of
                                Success doc ->
                                    ( { model
                                        | workError = Nothing
                                        , workBusy = False
                                        , commentForm = emptyCommentForm
                                        , replyingToCommentId = Nothing
                                        , editingCommentId = Nothing
                                      }
                                    , Api.listCommentsForDocument doc.id GotComments
                                    )

                                _ ->
                                    ( { model | workBusy = False, commentForm = emptyCommentForm }, Cmd.none )

                        _ ->
                            ( { model | workBusy = False, commentForm = emptyCommentForm }, Cmd.none )
                Err err ->
                    setWorkError ("Failed to add comment: " ++ httpErrorToString err) model

        -- Inline editing
        StartEditField field ->
            ( { model | editingField = field }, Cmd.none )

        CancelEditField ->
            ( { model | editingField = NotEditing }, Cmd.none )

        SaveEditField ->
            case model.workTask of
                Success task ->
                    let
                        ( updateFields, shouldSave ) =
                            case model.editingField of
                                EditingTitle val ->
                                    ( { title = Just val, description = Nothing, status = Nothing, priority = Nothing, tags = Nothing, comment = Nothing }, not (String.isEmpty (String.trim val)) )

                                EditingDescription val ->
                                    ( { title = Nothing, description = Just val, status = Nothing, priority = Nothing, tags = Nothing, comment = Nothing }, True )

                                EditingPriority ->
                                    -- Priority is saved immediately via ChangeTaskPriority, not here
                                    ( { title = Nothing, description = Nothing, status = Nothing, priority = Nothing, tags = Nothing, comment = Nothing }, False )

                                EditingTags val ->
                                    let
                                        tags =
                                            val
                                                |> String.split ","
                                                |> List.map String.trim
                                                |> List.filter (not << String.isEmpty)
                                    in
                                    ( { title = Nothing, description = Nothing, status = Nothing, priority = Nothing, tags = Just tags, comment = Nothing }, True )

                                NotEditing ->
                                    ( { title = Nothing, description = Nothing, status = Nothing, priority = Nothing, tags = Nothing, comment = Nothing }, False )
                    in
                    if shouldSave then
                        ( { model | workBusy = True, editingField = NotEditing }
                        , Api.updateTask task.id updateFields GotTaskUpdated
                        )
                    else
                        ( { model | editingField = NotEditing }, Cmd.none )

                _ ->
                    ( { model | editingField = NotEditing }, Cmd.none )

        -- Board view
        ToggleTaskViewMode ->
            ( { model
                | taskViewMode =
                    case model.taskViewMode of
                        ListView -> BoardView
                        BoardView -> ListView
              }
            , Cmd.none
            )

        DragStart taskId ->
            ( { model | draggingTaskId = Just taskId }, Cmd.none )

        DragEnd ->
            ( { model | draggingTaskId = Nothing }, Cmd.none )

        DragOver ->
            ( model, Cmd.none )

        DropOnStatus newStatus ->
            let
                resultMsg =
                    if model.route == LiveBoardRoute then GotLiveBoardTaskUpdated else GotBoardTaskUpdated
            in
            case model.draggingTaskId of
                Just taskId ->
                    if newStatus == "ready_for_review" then
                        ( { model | draggingTaskId = Nothing, boardDropTarget = Just ( taskId, newStatus ), boardDropComment = "" }, Cmd.none )
                    else
                        ( { model | draggingTaskId = Nothing, workBusy = True }
                        , Api.updateTask taskId
                            { title = Nothing, description = Nothing, status = Just newStatus, priority = Nothing, tags = Nothing, comment = Nothing }
                            resultMsg
                        )

                Nothing ->
                    ( model, Cmd.none )

        BoardDropCommentChange val ->
            ( { model | boardDropComment = val }, Cmd.none )

        SubmitBoardDrop ->
            let
                resultMsg =
                    if model.route == LiveBoardRoute then GotLiveBoardTaskUpdated else GotBoardTaskUpdated
            in
            case model.boardDropTarget of
                Just ( taskId, newStatus ) ->
                    let
                        comment = String.trim model.boardDropComment
                    in
                    if String.isEmpty comment then
                        setWorkError "Comment is required when moving to ready for review." model
                    else
                        ( { model | workBusy = True, boardDropTarget = Nothing, boardDropComment = "" }
                        , Api.updateTask taskId
                            { title = Nothing, description = Nothing, status = Just newStatus, priority = Nothing, tags = Nothing, comment = Just comment }
                            resultMsg
                        )

                Nothing ->
                    ( model, Cmd.none )

        CancelBoardDrop ->
            ( { model | boardDropTarget = Nothing, boardDropComment = "" }, Cmd.none )

        -- Filters
        ToggleStatusFilter val ->
            let
                f = model.taskFilters
                newFilter =
                    if List.member val f.statusFilter then
                        List.filter (\s -> s /= val) f.statusFilter
                    else
                        val :: f.statusFilter
            in
            ( { model | taskFilters = { f | statusFilter = newFilter } }, Cmd.none )

        ClearFilters ->
            ( { model | taskFilters = emptyTaskFilters }, Cmd.none )

        -- Project detail tabs
        ChangeProjectTab tab ->
            ( { model | projectTab = tab }, Cmd.none )

        -- Live board actions
        SelectLiveBoardTask taskId ->
            ( { model | workBusy = True }
            , Api.selectTasks [ taskId ] GotTaskSelected
            )

        DeselectLiveBoardTask taskId ->
            ( { model | workBusy = True }
            , Api.deselectTask taskId GotTaskDeselected
            )

        MoveLiveSelection taskId position ->
            ( { model | workBusy = True }, Api.moveSelection taskId position GotSelectionMoved )

        GotSelectionMoved result ->
            case result of
                Ok _ ->
                    ( { model | workError = Nothing, workBusy = False }, Api.getLiveBoard GotLiveBoard )

                Err err ->
                    setWorkError ("Failed to move selection: " ++ httpErrorToString err) model

        TakeNextTaskAction ->
            case model.workProject of
                Success proj ->
                    ( { model | workBusy = True }, Api.takeNextTask proj.id False GotTakeNextTaskAction )

                _ ->
                    ( model, Cmd.none )

        GotTakeNextTaskAction result ->
            case result of
                Ok _ ->
                    workSuccess model (refreshCurrentProjectTasksAndAnalytics model)

                Err err ->
                    setWorkError ("Take-next failed: " ++ httpErrorToString err) model

        TakeNextReviewTaskAction ->
            case model.workProject of
                Success proj ->
                    ( { model | workBusy = True }, Api.takeNextReviewTask proj.id False GotTakeNextReviewTaskAction )

                _ ->
                    ( model, Cmd.none )

        GotTakeNextReviewTaskAction result ->
            case result of
                Ok _ ->
                    workSuccess model (refreshCurrentProjectTasksAndAnalytics model)

                Err err ->
                    setWorkError ("Take-next-review failed: " ++ httpErrorToString err) model

        MoveTaskToTop ->
            case model.workTask of
                Success task ->
                    ( { model | workBusy = True }, Api.moveTaskToTopOrBottom task.id "top" GotTaskUpdated )

                _ ->
                    ( model, Cmd.none )

        MoveTaskToBottom ->
            case model.workTask of
                Success task ->
                    ( { model | workBusy = True }, Api.moveTaskToTopOrBottom task.id "bottom" GotTaskUpdated )

                _ ->
                    ( model, Cmd.none )

        RejectReviewCommentChange val ->
            ( { model | rejectReviewComment = val }, Cmd.none )

        RejectReviewAction ->
            case model.workTask of
                Success task ->
                    let
                        comment =
                            String.trim model.rejectReviewComment
                    in
                    if String.isEmpty comment then
                        setWorkError "Rejection comment is required." model
                    else
                        ( { model | workBusy = True }, Api.rejectReview task.id comment GotTaskUpdated )

                _ ->
                    ( model, Cmd.none )

        ReadyForReviewCommentChange val ->
            ( { model | readyForReviewComment = val }, Cmd.none )

        SubmitReadyForReview ->
            case model.workTask of
                Success task ->
                    let
                        comment =
                            String.trim model.readyForReviewComment
                    in
                    if String.isEmpty comment then
                        setWorkError "Comment is required when moving to ready for review." model
                    else
                        ( { model | workBusy = True, pendingStatusChange = Nothing }
                        , Api.updateTask task.id
                            { title = Nothing, description = Nothing, status = Just "ready_for_review", priority = Nothing, tags = Nothing, comment = Just comment }
                            GotTaskUpdated
                        )

                _ ->
                    ( model, Cmd.none )

        CancelPendingStatus ->
            ( { model | pendingStatusChange = Nothing, readyForReviewComment = "" }, Cmd.none )

        GotTaskSelected result ->
            case result of
                Ok _ ->
                    workSuccess model (Api.getLiveBoard GotLiveBoard)
                Err err ->
                    setWorkError ("Failed to select task: " ++ httpErrorToString err) model

        GotTaskDeselected result ->
            case result of
                Ok _ ->
                    workSuccess model (Api.getLiveBoard GotLiveBoard)
                Err err ->
                    setWorkError ("Failed to deselect task: " ++ httpErrorToString err) model

        ClearCompletedTasks ->
            ( { model | workBusy = True }, Api.clearCompleted GotClearedCompleted )

        GotClearedCompleted result ->
            case result of
                Ok _ ->
                    workSuccess model (Api.getLiveBoard GotLiveBoard)
                Err err ->
                    setWorkError ("Failed to clear completed: " ++ httpErrorToString err) model

        EnsureAgentLoopAction ->
            ( { model | workBusy = True }, Api.startAgentLoop GotAgentLoopAction )

        StopAgentLoopAction ->
            ( { model | workBusy = True }, Api.stopAgentLoop GotAgentLoopAction )

        GotAgentLoopAction result ->
            case result of
                Ok message ->
                    setWorkNotice message { model | workBusy = False, workError = Nothing }

                Err err ->
                    setWorkError ("Agent action failed: " ++ httpErrorToString err) model

        -- ============ Voice Messages ============

        VoiceStartRecording mode ->
            let
                vs = model.voiceState
                existingContent =
                    case ( mode, model.workTask ) of
                        ( VoiceEdit, Success task ) ->
                            Just task.description

                        _ ->
                            vs.existingContent
            in
            ( { model | voiceState = { vs | recordingState = VoiceRecording, mode = mode, existingContent = existingContent } }
            , startRecording ()
            )

        VoiceStopRecording ->
            ( model, stopRecording False )

        VoiceGotAudio base64Data mimeType ->
            if String.isEmpty base64Data then
                let
                    vs = model.voiceState
                in
                ( { model | voiceState = { vs | recordingState = VoiceError "Recording produced no audio data" } }
                , Cmd.none
                )
            else
            case model.chatState.activeChatId of
                Just cid ->
                    let
                        convState = getConversationState cid model.chatState
                        voiceFilename = "voice_message" ++ extensionForMime mimeType
                    in
                    if isRecording convState.activity then
                        let
                            payload = UploadVoice { data = base64Data, mimeType = mimeType }
                            ( uploadId, newConvState ) = enqueueUpload payload { convState | activity = ChatIdle }
                            newChatState = updateConversationState cid (\_ -> newConvState) model.chatState
                        in
                        ( { model | chatState = newChatState }
                        , Api.uploadChatMedia cid uploadId base64Data voiceFilename mimeType (GotChatMediaUploaded cid)
                        )

                    else
                        case model.voiceState.recordingState of
                            VoiceRecording ->
                                let
                                    vs = model.voiceState
                                in
                                ( { model | voiceState = { vs | recordingState = VoiceTranscribing } }
                                , Api.transcribeAudio base64Data mimeType GotTranscription
                                )

                            _ ->
                                -- Stale recording — context was destroyed, ignore
                                ( model, Cmd.none )

                Nothing ->
                    -- Work voice flow
                    case model.voiceState.recordingState of
                        VoiceRecording ->
                            let
                                vs = model.voiceState
                            in
                            ( { model | voiceState = { vs | recordingState = VoiceTranscribing } }
                            , Api.transcribeAudio base64Data mimeType GotTranscription
                            )

                        _ ->
                            -- Stale recording — context was destroyed, ignore
                            ( model, Cmd.none )

        VoiceRecordingFailed errorMsg ->
            let
                vs = model.voiceState
                newChatState =
                    case model.chatState.activeChatId of
                        Just cid ->
                            updateConversationState cid
                                (\conv ->
                                    if isRecording conv.activity then
                                        { conv | activity = ChatError { error = MediaUploadFailed errorMsg, retryable = False, failedContent = Nothing } }
                                    else
                                        conv
                                )
                                model.chatState

                        Nothing ->
                            model.chatState
            in
            ( { model
                | voiceState = { vs | recordingState = VoiceError errorMsg }
                , chatState = newChatState
              }
            , Cmd.none
            )

        GotTranscription result ->
            case result of
                Ok transcribeResult ->
                    if transcribeResult.success then
                        case transcribeResult.transcription of
                            Just text ->
                                let
                                    vs = model.voiceState
                                    newVs = { vs | recordingState = VoiceFormatting, transcription = Just text }
                                in
                                ( { model | voiceState = newVs }
                                , Api.formatTranscription text (voiceModeToString vs.mode) vs.existingContent GotFormatted
                                )

                            Nothing ->
                                let
                                    vs = model.voiceState
                                in
                                ( { model | voiceState = { vs | recordingState = VoiceError "No transcription returned" } }
                                , Cmd.none
                                )
                    else
                        let
                            vs = model.voiceState
                            errMsg = Maybe.withDefault "Transcription failed" transcribeResult.error
                        in
                        ( { model | voiceState = { vs | recordingState = VoiceError errMsg } }
                        , Cmd.none
                        )

                Err err ->
                    let
                        vs = model.voiceState
                    in
                    ( { model | voiceState = { vs | recordingState = VoiceError (httpErrorToString err) } }
                    , Cmd.none
                    )

        GotFormatted result ->
            case result of
                Ok formatResult ->
                    if formatResult.success then
                        let
                            vs = model.voiceState
                            formatted = Maybe.withDefault "" formatResult.formatted
                        in
                        ( { model | voiceState = { vs | recordingState = VoiceDone formatted } }
                        , Cmd.none
                        )
                    else
                        let
                            vs = model.voiceState
                            errMsg = Maybe.withDefault "Formatting failed" formatResult.error
                        in
                        ( { model | voiceState = { vs | recordingState = VoiceError errMsg } }
                        , Cmd.none
                        )

                Err err ->
                    let
                        vs = model.voiceState
                    in
                    ( { model | voiceState = { vs | recordingState = VoiceError (httpErrorToString err) } }
                    , Cmd.none
                    )

        VoiceReset ->
            ( { model | voiceState = emptyVoiceState }, Cmd.none )

        VoiceSetExistingContent content ->
            let
                vs = model.voiceState
            in
            ( { model | voiceState = { vs | existingContent = Just content } }, Cmd.none )

        VoiceApplyToTaskForm ->
            case model.voiceState.recordingState of
                VoiceDone formatted ->
                    let
                        ( extractedTitle, extractedDesc ) = extractTitleAndDescription formatted
                        form = model.taskForm
                    in
                    ( { model
                        | taskForm = { form | title = extractedTitle, description = extractedDesc }
                        , voiceState = emptyVoiceState
                      }
                    , Cmd.none
                    )

                _ ->
                    ( model, Cmd.none )

        VoiceApplyToTaskEdit ->
            case ( model.voiceState.recordingState, model.workTask ) of
                ( VoiceDone formatted, Success task ) ->
                    ( { model
                        | workBusy = True
                        , voiceState = emptyVoiceState
                      }
                    , Api.updateTask task.id
                        { title = Nothing
                        , description = Just formatted
                        , status = Nothing
                        , priority = Nothing
                        , tags = Nothing
                        , comment = Nothing
                        }
                        GotTaskUpdated
                    )

                _ ->
                    ( model, Cmd.none )

        VoiceApplyToComment ->
            case model.voiceState.recordingState of
                VoiceDone formatted ->
                    let
                        form = model.commentForm
                    in
                    ( { model
                        | commentForm = { form | content = formatted }
                        , voiceState = emptyVoiceState
                      }
                    , Cmd.none
                    )

                _ ->
                    ( model, Cmd.none )

        VoiceEditComment commentId ->
            let
                existingContent =
                    case model.workComments of
                        Success cmts ->
                            cmts
                                |> List.filter (\c -> c.id == commentId)
                                |> List.head
                                |> Maybe.map .content

                        _ ->
                            Nothing

                vs = model.voiceState
            in
            ( { model
                | editingCommentId = Just commentId
                , commentForm = { content = Maybe.withDefault "" existingContent }
                , voiceState = { vs | mode = VoiceEdit, existingContent = existingContent, recordingState = VoiceRecording }
              }
            , startRecording ()
            )

        -- ═══════════════════════════════════════════════════════════════
        -- CHAT handlers
        -- ═══════════════════════════════════════════════════════════════

        GotConversations result ->
            let
                cs = model.chatState
                newCs = { cs | conversations = resultToRemote result }

                -- If viewing a telegram conversation, set ChatObserving
                withObserving =
                    case cs.activeChatId of
                        Just cid ->
                            if isTelegramConversation cid newCs then
                                updateConversationState cid
                                    (\conv ->
                                        case conv.activity of
                                            ChatIdle -> { conv | activity = ChatObserving }
                                            _ -> conv
                                    )
                                    newCs
                            else
                                newCs

                        Nothing ->
                            newCs
            in
            ( { model | chatState = withObserving }, Cmd.none )

        GotChatMessages convId result ->
            let
                cs = model.chatState
            in
            case result of
                Ok msgs ->
                    let
                        serverIds = List.map .id msgs
                    in
                    ( { model
                        | chatState =
                            updateConversationState convId
                                (\conv ->
                                    { conv
                                        | messages = msgs
                                        , pendingOutbound =
                                            List.filter
                                                (\p -> not (List.member p.id serverIds))
                                                conv.pendingOutbound
                                        , messagesLoaded = MessagesLoaded { hasMore = List.length msgs >= 50 }
                                    }
                                )
                                cs
                      }
                    , forceScrollToBottom "chat-messages"
                    )

                Err e ->
                    ( { model
                        | chatState =
                            updateConversationState convId
                                (\conv -> { conv | messagesLoaded = MessagesLoadError (httpErrorToString e) })
                                cs
                      }
                    , Cmd.none
                    )

        ChatNewConversation ->
            ( model, Api.createConversation GotConversationCreated )

        GotConversationCreated result ->
            case result of
                Ok resp ->
                    ( model, Nav.pushUrl model.navKey (routeToUrl (ChatRoute (Just resp.conversationId))) )

                Err _ ->
                    ( model, Cmd.none )

        ChatSelectConversation cid ->
            if canSwitchConversation model.chatState then
                ( model, Nav.pushUrl model.navKey (routeToUrl (ChatRoute (Just cid))) )
            else
                ( model, Cmd.none )

        ChatInputChange text ->
            case model.chatState.activeChatId of
                Just cid ->
                    let
                        cs = model.chatState
                    in
                    ( { model
                        | chatState =
                            updateConversationState cid
                                (\conv ->
                                    let
                                        newActivity =
                                            case conv.activity of
                                                ChatIdle ->
                                                    if String.isEmpty (String.trim text) then ChatIdle
                                                    else ChatComposing ComposingText

                                                ChatComposing ComposingText ->
                                                    if String.isEmpty (String.trim text) then ChatIdle
                                                    else ChatComposing ComposingText

                                                other ->
                                                    other
                                    in
                                    { conv | inputText = text, activity = newActivity }
                                )
                                cs
                      }
                    , Cmd.none
                    )

                Nothing ->
                    ( model, Cmd.none )

        ChatSendMessage ->
            case model.chatState.activeChatId of
                Just cid ->
                    let
                        cs = model.chatState
                        conv = getConversationState cid cs
                        content = String.trim conv.inputText
                    in
                    if String.isEmpty content || not (canSendMessage conv.activity) then
                        ( model, Cmd.none )
                    else
                        let
                            ( msgId, conv2 ) = nextMessageId conv
                            userMsg =
                                { id = msgId
                                , direction = Inbound
                                , content = content
                                , timestamp = ""
                                , attachments = []
                                }
                            conv3 =
                                { conv2
                                    | inputText = ""
                                    , activity = ChatSending (SendingText { content = content, pendingId = msgId })
                                    , pendingOutbound = conv2.pendingOutbound ++ [ userMsg ]
                                }
                        in
                        ( { model | chatState = updateConversationState cid (\_ -> conv3) cs }
                        , Cmd.batch
                            [ Api.sendChatMessage cid content (GotChatMessageSent cid)
                            , forceScrollToBottom "chat-messages"
                            ]
                        )

                Nothing ->
                    ( model, Cmd.none )

        GotChatMessageSent convId result ->
            let
                cs = model.chatState
            in
            case result of
                Ok resp ->
                    ( { model
                        | chatState =
                            updateConversationState convId
                                (\conv ->
                                    case conv.activity of
                                        ChatSending (SendingText payload) ->
                                            { conv
                                                | activity = ChatAwaitingResponse { pendingId = payload.pendingId }
                                                , pendingOutbound =
                                                    List.map
                                                        (\p ->
                                                            if p.id == payload.pendingId then
                                                                { p | id = resp.messageId }
                                                            else
                                                                p
                                                        )
                                                        conv.pendingOutbound
                                            }

                                        _ ->
                                            conv
                                )
                                cs
                      }
                    , Cmd.none
                    )

                Err e ->
                    let
                        errInfo = { error = SendFailed (httpErrorToString e), retryable = True, failedContent = Nothing }
                        newChatState =
                            updateConversationState convId
                                (\conv ->
                                    case conv.activity of
                                        ChatSending (SendingText payload) ->
                                            { conv
                                                | activity = ChatError { errInfo | failedContent = Just payload.content }
                                                , pendingOutbound = List.filter (\p -> p.id /= payload.pendingId) conv.pendingOutbound
                                            }

                                        _ ->
                                            { conv | activity = ChatError errInfo }
                                )
                                cs
                    in
                    if isBackground convId cs then
                        let
                            withNotif = addNotification convId (ChatErrorNotification (SendFailed (httpErrorToString e))) newChatState
                        in
                        ( { model | chatState = withNotif }
                        , autoDismissNotification withNotif.notificationCounter
                        )
                    else
                        ( { model | chatState = newChatState }, Cmd.none )

        ChatWsReceived rawData ->
            case decodeWsEvent rawData of
                Ok (WsMessageChunk convId content seq isFinal) ->
                    let
                        cs = model.chatState
                        conv = getConversationState convId cs
                    in
                    case conv.activity of
                        ChatIdle ->
                            -- Already finalized — ignore duplicate/stale event
                            ( model, Cmd.none )

                        _ ->
                            if isFinal then
                                let
                                    newChatState =
                                        updateConversationState convId
                                            (\c ->
                                                let
                                                    ( msgId, c2 ) = nextMessageId c
                                                    
                                                    -- Important: if the string is completely empty and it's final,
                                                    -- it might mean the LLM finished answering using tools and had 
                                                    -- no text response. But typically there's SOME buffer.
                                                    finalContent =
                                                        case c.activity of
                                                            ChatStreaming s -> s.buffer ++ content
                                                            _ -> content

                                                    finalMsg =
                                                        { id = msgId
                                                        , direction = Outbound
                                                        , content = finalContent
                                                        , timestamp = ""
                                                        , attachments = []
                                                        }
                                                in
                                                { c2
                                                    | activity = ChatIdle
                                                    , messages = c2.messages ++ c2.pendingOutbound ++ (if String.isEmpty finalContent then [] else [ finalMsg ])
                                                    , pendingOutbound = []
                                                    , lastChunkSequence = seq
                                                }
                                            )
                                            cs

                                    ( withNotification, notifCmd ) =
                                        if isBackground convId cs then
                                            let
                                                preview = String.left 80 content
                                                ns = addNotification convId (ResponseComplete { preview = preview }) newChatState
                                            in
                                            ( ns, autoDismissNotification ns.notificationCounter )
                                        else
                                            ( newChatState, Cmd.none )
                                in
                                ( { model | chatState = withNotification }
                                , Cmd.batch [ scrollToBottom "chat-messages", notifCmd ]
                                )
                            else
                                ( { model
                                    | chatState =
                                        updateConversationState convId
                                            (\c ->
                                                let
                                                    newBuffer =
                                                        case c.activity of
                                                            ChatStreaming s -> s.buffer ++ content
                                                            _ -> content
                                                in
                                                { c
                                                    | activity = ChatStreaming { buffer = newBuffer }
                                                    , lastChunkSequence = seq
                                                }
                                            )
                                            cs
                                  }
                                , scrollToBottom "chat-messages"
                                )

                Ok (WsConversationRenamed convId name) ->
                    let
                        cs = model.chatState
                        updateConv c =
                            if c.id == convId then
                                { c | name = name, autoName = Just name }
                            else
                                c

                        newConversations =
                            case cs.conversations of
                                Success convs ->
                                    Success (List.map updateConv convs)

                                other ->
                                    other
                    in
                    ( { model | chatState = { cs | conversations = newConversations } }, Cmd.none )

                Ok (WsTranscribing convId) ->
                    let
                        cs = model.chatState
                        conv = getConversationState convId cs
                    in
                    case conv.activity of
                        ChatAwaitingResponse _ ->
                            ( model, Cmd.none )

                        ChatStreaming _ ->
                            ( model, Cmd.none )

                        ChatTranscribing ->
                            ( model, Cmd.none )

                        _ ->
                            ( { model
                                | chatState =
                                    updateConversationState convId
                                        (\c -> { c | activity = ChatTranscribing })
                                        cs
                              }
                            , scrollToBottom "chat-messages"
                            )

                Ok (WsTypingIndicator convId) ->
                    let
                        cs = model.chatState
                        conv = getConversationState convId cs
                    in
                    case conv.activity of
                        ChatAwaitingResponse _ ->
                            ( model, Cmd.none )

                        ChatStreaming _ ->
                            ( model, Cmd.none )

                        _ ->
                            ( { model
                                | chatState =
                                    updateConversationState convId
                                        (\c -> { c | activity = ChatAwaitingResponse { pendingId = "" } })
                                        cs
                              }
                            , scrollToBottom "chat-messages"
                            )

                Ok (WsMessageUpdated convId msgId newContent) ->
                    ( { model
                        | chatState =
                            updateConversationState convId
                                (\conv ->
                                    let
                                        updateMsg m =
                                            if m.id == msgId then
                                                { m | content = newContent }
                                            else
                                                m
                                    in
                                    { conv
                                        | messages = List.map updateMsg conv.messages
                                        , pendingOutbound = List.map updateMsg conv.pendingOutbound
                                        -- Important: We do not clear the ChatAwaitingResponse/ChatStreaming state here. 
                                        -- The message is simply updated (appended to). The state remains Streaming 
                                        -- until WsChunkReceived marks it as final, or an error occurs.
                                    }
                                )
                                model.chatState
                      }
                    , scrollToBottom "chat-messages"
                    )

                Ok (WsFileMessage info) ->
                    let
                        cs = model.chatState

                        attachment =
                            if String.startsWith "image" info.mimeType then
                                ImageAttachment { path = info.mediaPath, description = Nothing }
                            else
                                FileAttachment { path = info.mediaPath, name = info.filename, mimeType = info.mimeType }

                        fileMsg =
                            { id = info.messageId
                            , direction = Outbound
                            , content = info.caption
                            , timestamp = ""
                            , attachments = [ attachment ]
                            }

                        newChatState =
                            updateConversationState info.conversationId
                                (\c -> { c | messages = c.messages ++ [ fileMsg ], activity = ChatIdle })
                                cs
                    in
                    ( { model | chatState = newChatState }
                    , scrollToBottom "chat-messages"
                    )

                Ok (WsConnectionState convId connState) ->
                    ( { model
                        | chatState =
                            updateConversationState convId
                                (\conv -> { conv | connection = connState })
                                model.chatState
                      }
                    , Cmd.none
                    )

                Err decodeError ->
                    ( model, logWarning ("WS decode error: " ++ decodeError) )

        ChatStartRename convId currentName ->
            let
                cs = model.chatState
            in
            ( { model | chatState = { cs | renamingConversationId = Just convId, renameText = currentName } }, Cmd.none )

        ChatRenameChange text ->
            let
                cs = model.chatState
            in
            ( { model | chatState = { cs | renameText = text } }, Cmd.none )

        ChatSubmitRename ->
            let
                cs = model.chatState
            in
            case cs.renamingConversationId of
                Just convId ->
                    let
                        newName = String.trim cs.renameText
                    in
                    if String.isEmpty newName then
                        ( model, Cmd.none )

                    else
                        let
                            updateConv c =
                                if c.id == convId then
                                    { c | name = newName, customName = Just newName }
                                else
                                    c

                            newConversations =
                                case cs.conversations of
                                    Success convs ->
                                        Success (List.map updateConv convs)

                                    other ->
                                        other
                        in
                        ( { model | chatState = { cs | renamingConversationId = Nothing, renameText = "", conversations = newConversations } }
                        , Api.renameConversation convId newName GotChatRenamed
                        )

                Nothing ->
                    ( model, Cmd.none )

        ChatCancelRename ->
            let
                cs = model.chatState
            in
            ( { model | chatState = { cs | renamingConversationId = Nothing, renameText = "" } }, Cmd.none )

        GotChatRenamed _ ->
            ( model, Cmd.none )

        ChatConfirmDelete convId ->
            let
                cs = model.chatState
            in
            ( { model | chatState = { cs | confirmingDeleteId = Just convId } }, Cmd.none )

        ChatCancelDelete ->
            let
                cs = model.chatState
            in
            ( { model | chatState = { cs | confirmingDeleteId = Nothing } }, Cmd.none )

        ChatDeleteConversation convId ->
            let
                cs = model.chatState
            in
            ( { model | chatState = { cs | confirmingDeleteId = Nothing } }
            , Api.deleteConversation convId (GotChatDeleted convId)
            )

        GotChatDeleted deletedId result ->
            case result of
                Ok () ->
                    let
                        cs = model.chatState

                        newConversations =
                            case cs.conversations of
                                Success convs ->
                                    Success (List.filter (\c -> c.id /= deletedId) convs)

                                other ->
                                    other

                        needsNavigate =
                            cs.activeChatId == Just deletedId

                        cleanedStates =
                            { cs
                                | conversations = newConversations
                                , conversationStates = Dict.remove deletedId cs.conversationStates
                            }
                    in
                    ( { model | chatState = cleanedStates }
                    , Cmd.batch
                        [ unsubscribeChatWS deletedId
                        , if needsNavigate then
                            Nav.pushUrl model.navKey (routeToUrl (ChatRoute Nothing))
                          else
                            Cmd.none
                        ]
                    )

                Err _ ->
                    ( model, Cmd.none )

        ChatStartVoice ->
            case model.chatState.activeChatId of
                Just cid ->
                    let
                        cs = model.chatState
                        conv = getConversationState cid cs
                    in
                    if canStartRecording conv.activity then
                        ( { model
                            | chatState =
                                updateConversationState cid
                                    (\c -> { c | activity = ChatComposing ComposingVoice })
                                    cs
                          }
                        , startRecording ()
                        )
                    else
                        ( model, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        ChatStopVoice ->
            -- Don't set ChatIdle here -- VoiceGotAudio will transition
            -- after the async JS onstop callback delivers the audio data
            ( model, stopRecording False )

        ChatCancelVoice ->
            case model.chatState.activeChatId of
                Just cid ->
                    ( { model
                        | chatState =
                            updateConversationState cid
                                (\conv -> { conv | activity = ChatIdle })
                                model.chatState
                      }
                    , stopRecording True
                    )

                Nothing ->
                    ( model, stopRecording True )

        ChatAttachFile ->
            case model.chatState.activeChatId of
                Just cid ->
                    let
                        conv = getConversationState cid model.chatState
                    in
                    if canAttachFile conv.activity then
                        ( model, triggerFileInput () )
                    else
                        ( model, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        ChatFileReceived jsonValue ->
            case model.chatState.activeChatId of
                Just cid ->
                    case D.decodeValue chatFileDecoder jsonValue of
                        Ok fileData ->
                            let
                                cs = model.chatState
                                conv = getConversationState cid cs
                                payload = UploadFile { data = fileData.data, name = fileData.name, mimeType = fileData.mimeType }
                                ( uploadId, newConv ) = enqueueUpload payload conv
                            in
                            ( { model | chatState = updateConversationState cid (\_ -> newConv) cs }
                            , Api.uploadChatMedia cid uploadId fileData.data fileData.name fileData.mimeType (GotChatMediaUploaded cid)
                            )

                        Err _ ->
                            ( model, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        ChatStartVideo ->
            case model.chatState.activeChatId of
                Just cid ->
                    let
                        cs = model.chatState
                        conv = getConversationState cid cs
                    in
                    if canStartRecording conv.activity then
                        ( { model
                            | chatState =
                                updateConversationState cid
                                    (\c -> { c | activity = ChatComposing ComposingVideo })
                                    cs
                          }
                        , startVideoRecording ()
                        )
                    else
                        ( model, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        ChatStopVideo ->
            -- Don't set ChatIdle here -- ChatVideoReceived will transition
            -- after the async JS onstop callback delivers the video data
            ( model, stopVideoRecording False )

        ChatCancelVideo ->
            case model.chatState.activeChatId of
                Just cid ->
                    ( { model
                        | chatState =
                            updateConversationState cid
                                (\conv -> { conv | activity = ChatIdle })
                                model.chatState
                      }
                    , stopVideoRecording True
                    )

                Nothing ->
                    ( model, stopVideoRecording True )

        ChatVideoReceived jsonValue ->
            case model.chatState.activeChatId of
                Just cid ->
                    case D.decodeValue chatVideoDecoder jsonValue of
                        Ok videoData ->
                            let
                                cs = model.chatState
                                conv = getConversationState cid cs
                                payload = UploadVideo { data = videoData.data, mimeType = videoData.mimeType }
                                ( uploadId, newConv ) = enqueueUpload payload { conv | activity = ChatIdle }
                            in
                            ( { model | chatState = updateConversationState cid (\_ -> newConv) cs }
                            , Api.uploadChatMedia cid uploadId videoData.data "video_message.webm" videoData.mimeType (GotChatMediaUploaded cid)
                            )

                        Err _ ->
                            ( { model
                                | chatState =
                                    updateConversationState cid
                                        (\conv -> { conv | activity = ChatIdle })
                                        model.chatState
                              }
                            , Cmd.none
                            )

                Nothing ->
                    ( model, Cmd.none )

        GotChatMediaUploaded cid uploadId result ->
            let
                cs = model.chatState
            in
            case result of
                Ok response ->
                    if response.success then
                        let
                            displayText =
                                Maybe.withDefault "[Media uploaded]" response.transcription

                            attachments =
                                mediaAttachmentFromServerFields response.mediaType response.mediaPath

                            newChatState =
                                updateConversationState cid
                                    (\conv ->
                                        let
                                            ( msgId, conv2 ) = nextMessageId conv
                                            conv3 = removeUpload uploadId conv2
                                            userMsg =
                                                { id = Maybe.withDefault msgId response.messageId
                                                , direction = Inbound
                                                , content = displayText
                                                , timestamp = ""
                                                , attachments = attachments
                                                }
                                        in
                                        { conv3
                                            | pendingOutbound = conv3.pendingOutbound ++ [ userMsg ]
                                        }
                                    )
                                    cs

                            ( withNotification, notifCmd ) =
                                if isBackground cid cs then
                                    let
                                        ns = addNotification cid MediaUploadComplete newChatState
                                    in
                                    ( ns, autoDismissNotification ns.notificationCounter )
                                else
                                    ( newChatState, Cmd.none )
                        in
                        ( { model | chatState = withNotification }
                        , Cmd.batch [ scrollToBottom "chat-messages", notifCmd ]
                        )

                    else
                        let
                            errMsg = Maybe.withDefault "Upload failed" response.error
                        in
                        ( { model
                            | chatState =
                                updateConversationState cid
                                    (\conv -> updateUpload uploadId (UploadFailed errMsg) conv)
                                    cs
                          }
                        , Cmd.none
                        )

                Err e ->
                    ( { model
                        | chatState =
                            updateConversationState cid
                                (\conv -> updateUpload uploadId (UploadFailed (httpErrorToString e)) conv)
                                cs
                      }
                    , Cmd.none
                    )

        ChatDismissError convId ->
            ( { model
                | chatState =
                    updateConversationState convId
                        (\conv -> { conv | activity = ChatIdle })
                        model.chatState
              }
            , Cmd.none
            )

        ChatDismissNotification noteId ->
            ( { model | chatState = dismissNotification noteId model.chatState }, Cmd.none )

        ChatRetryUpload convId uploadId ->
            let
                cs = model.chatState
                conv = getConversationState convId cs
                maybeTask = List.filter (\u -> u.id == uploadId) conv.uploads |> List.head
            in
            case maybeTask of
                Just task ->
                    let
                        ( data, filename, mimeType ) =
                            case task.media of
                                UploadVoice v -> ( v.data, "voice_message.webm", v.mimeType )
                                UploadVideo v -> ( v.data, "video_message.webm", v.mimeType )
                                UploadFile f -> ( f.data, f.name, f.mimeType )

                        newChatState =
                            updateConversationState convId
                                (\c -> updateUpload uploadId Uploading c)
                                cs
                    in
                    ( { model | chatState = newChatState }
                    , Api.uploadChatMedia convId uploadId data filename mimeType (GotChatMediaUploaded convId)
                    )

                Nothing ->
                    ( model, Cmd.none )


        ChatPollTelegram convId ->
            ( model, Api.getChatMessages convId (GotChatMessages convId) )


-- File/Video data decoders for ports


type alias ChatFileData =
    { name : String
    , mimeType : String
    , data : String
    }


chatFileDecoder : D.Decoder ChatFileData
chatFileDecoder =
    D.map3 ChatFileData
        (D.field "name" D.string)
        (D.field "mime_type" D.string)
        (D.field "data" D.string)


type alias ChatVideoData =
    { data : String
    , mimeType : String
    }


chatVideoDecoder : D.Decoder ChatVideoData
chatVideoDecoder =
    D.map2 ChatVideoData
        (D.field "data" D.string)
        (D.field "mime_type" D.string)


autoDismissNotification : Int -> Cmd Msg
autoDismissNotification noteId =
    Process.sleep 8000
        |> Task.perform (\_ -> ChatDismissNotification noteId)


-- WebSocket event parsing

type WsEvent
    = WsMessageChunk String String Int Bool
    | WsConversationRenamed String String
    | WsTypingIndicator String
    | WsTranscribing String
    | WsMessageUpdated String String String
    | WsFileMessage { conversationId : String, messageId : String, filename : String, mediaPath : String, mimeType : String, caption : String }
    | WsConnectionState String WsConnection


decodeWsEvent : String -> Result String WsEvent
decodeWsEvent raw =
    let
        decoder =
            D.field "type" D.string
                |> D.andThen (\t ->
                    case t of
                        "message_chunk" ->
                            D.map4 WsMessageChunk
                                (D.field "conversation_id" D.string)
                                (D.field "content" D.string)
                                (D.oneOf [ D.field "sequence" D.int, D.succeed 0 ])
                                (D.field "is_final" D.bool)

                        "conversation_renamed" ->
                            D.map2 WsConversationRenamed
                                (D.field "conversation_id" D.string)
                                (D.field "name" D.string)

                        "typing_indicator" ->
                            D.map WsTypingIndicator
                                (D.field "conversation_id" D.string)

                        "transcribing" ->
                            D.map WsTranscribing
                                (D.field "conversation_id" D.string)

                        "message_updated" ->
                            D.map3 WsMessageUpdated
                                (D.field "conversation_id" D.string)
                                (D.field "message_id" D.string)
                                (D.field "content" D.string)

                        "file_message" ->
                            D.field "conversation_id" D.string
                                |> D.andThen (\convId ->
                                    D.map5
                                        (\msgId fn mp mt cap ->
                                            WsFileMessage
                                                { conversationId = convId
                                                , messageId = msgId
                                                , filename = fn
                                                , mediaPath = mp
                                                , mimeType = mt
                                                , caption = cap
                                                }
                                        )
                                        (D.field "message_id" D.string)
                                        (D.field "filename" D.string)
                                        (D.field "media_path" D.string)
                                        (D.field "mime_type" D.string)
                                        (D.oneOf [ D.field "caption" D.string, D.succeed "" ])
                                )

                        "connection_state" ->
                            D.map2 WsConnectionState
                                (D.field "conversation_id" D.string)
                                (D.field "state" D.string
                                    |> D.andThen (\s ->
                                        case s of
                                            "connected" ->
                                                D.map WsConnected (D.field "conversation_id" D.string)

                                            "disconnected" ->
                                                D.succeed WsDisconnected

                                            "reconnecting" ->
                                                D.map2 WsReconnecting
                                                    (D.field "conversation_id" D.string)
                                                    (D.oneOf [ D.field "attempt" D.int, D.succeed 1 ])

                                            _ ->
                                                D.fail ("unknown connection state: " ++ s)
                                    )
                                )

                        _ ->
                            D.fail ("unknown WS event type: " ++ t)
                )
    in
    D.decodeString decoder raw
        |> Result.mapError D.errorToString


apiKeysResponseToData : Api.ApiKeysResponse -> ApiKeysData
apiKeysResponseToData resp =
    { hasTelegramToken = resp.hasTelegramToken
    , telegramTokenMasked = resp.telegramTokenMasked
    , telegramStatus = resp.telegramStatus
    , hasGeminiKey = resp.hasGeminiKey
    , geminiKeyMasked = resp.geminiKeyMasked
    , geminiStatus = resp.geminiStatus
    , claudeCodeStatus = resp.claudeCodeStatus
    , hasUserContacted = resp.hasUserContacted
    }


responseToSetupStatus : Api.SetupStatusResponse -> SetupStatus
responseToSetupStatus resp =
    { emptySetupStatus
        | dataDir = resp.dataDir
        , hasTelegramToken = resp.hasTelegramToken
        , hasGeminiKey = resp.hasGeminiKey
        , hasClaudeCli = resp.hasClaudeCli
        , claudeCliVersion = resp.claudeCliVersion
        , hasAllowedUsername = resp.hasAllowedUsername
        , isComplete = resp.isComplete
        , platform = resp.platform
        , hasThreadingEnabled = resp.hasThreadingEnabled
        , geminiKeyPreview = resp.geminiKeyPreview
        , allowedUsernameValue = resp.allowedUsernameValue
        , botName = resp.botName
    }


emptySetupStatus : SetupStatus
emptySetupStatus =
    { dataDir = ""
    , hasTelegramToken = False
    , hasGeminiKey = False
    , hasClaudeCli = False
    , claudeCliVersion = Nothing
    , hasAllowedUsername = False
    , isComplete = False
    , platform = ""
    , botName = Nothing
    , telegramError = Nothing
    , geminiError = Nothing
    , claudeInstalling = False
    , claudeInstallError = Nothing
    , allowedUsernameError = Nothing
    , claudeAuthenticated = False
    , claudeAuthMode = Nothing
    , claudeAccountEmail = Nothing
    , claudeAccountName = Nothing
    , claudeNeedsUpdate = False
    , claudeLatestVersion = Nothing
    , claudeUpdating = False
    , claudeUpdateError = Nothing
    , claudeTesting = False
    , claudeTestResult = Nothing
    , claudeTestOutput = Nothing
    , claudeTestError = Nothing
    , claudeAuthChecking = False
    , hasThreadingEnabled = False
    , threadingChecking = False
    , threadingError = Nothing
    , geminiKeyPreview = Nothing
    , allowedUsernameValue = Nothing
    }


currentSetupStatus : Model -> SetupStatus
currentSetupStatus model =
    withDefault emptySetupStatus model.setupStatus


sanitizeUsername : String -> String
sanitizeUsername raw =
    let
        trimmed = String.trim raw
    in
    if String.startsWith "@" trimmed then
        String.dropLeft 1 trimmed
    else
        trimmed


normalizeChatHarness : String -> String
normalizeChatHarness raw =
    case String.toLower (String.trim raw) of
        "codex" ->
            "codex"

        "echo" ->
            "echo"

        _ ->
            "claude"



setWorkError : String -> Model -> ( Model, Cmd Msg )
setWorkError err model =
    ( { model | workError = Just err, workBusy = False }
    , Process.sleep 5000 |> Task.perform (\_ -> ClearWorkError)
    )


setWorkNotice : String -> Model -> ( Model, Cmd Msg )
setWorkNotice msg model =
    ( { model | workNotice = Just msg }
    , Process.sleep 5000 |> Task.perform (\_ -> ClearWorkNotice)
    )


workSuccess : Model -> Cmd Msg -> ( Model, Cmd Msg )
workSuccess model cmd =
    ( { model | workError = Nothing, workBusy = False }, cmd )


refreshCurrentProjectTasksAndAnalytics : Model -> Cmd Msg
refreshCurrentProjectTasksAndAnalytics model =
    case model.workProject of
        Success proj ->
            Cmd.batch
                [ Api.listTasks (Just proj.id) Nothing GotWorkTasks
                , Api.getTaskAnalytics (Just proj.id) GotAnalytics
                ]

        _ ->
            Cmd.none


resultToRemote : Result Http.Error a -> RemoteData a
resultToRemote result =
    case result of
        Ok data -> Success data
        Err err -> Failure (httpErrorToString err)


{-| Resolve task/document context from the current route to build an upsertComment command.
    Returns Nothing if we're not on a task/document page or the resource isn't loaded.
-}
commentCmd : Model -> { commentId : Maybe Int, content : String, parentCommentId : Maybe Int } -> Maybe (Cmd Msg)
commentCmd model opts =
    case model.route of
        TaskDetailRoute _ ->
            case model.workTask of
                Success task ->
                    Just
                        (Api.upsertComment
                            { commentId = opts.commentId
                            , taskId = Just task.id
                            , documentId = Nothing
                            , content = opts.content
                            , parentCommentId = opts.parentCommentId
                            }
                            GotCommentCreated
                        )

                _ ->
                    Nothing

        DocumentDetailRoute _ ->
            case model.workDocument of
                Success doc ->
                    Just
                        (Api.upsertComment
                            { commentId = opts.commentId
                            , taskId = Nothing
                            , documentId = Just doc.id
                            , content = opts.content
                            , parentCommentId = opts.parentCommentId
                            }
                            GotCommentCreated
                        )

                _ ->
                    Nothing

        _ ->
            Nothing


httpErrorToString : Http.Error -> String
httpErrorToString err =
    case err of
        Http.BadUrl url -> "Bad URL: " ++ url
        Http.Timeout -> "Request timed out"
        Http.NetworkError -> "Network error"
        Http.BadStatus code -> "HTTP " ++ String.fromInt code
        Http.BadBody body -> "Bad response: " ++ body


extractTitleAndDescription : String -> ( String, String )
extractTitleAndDescription formatted =
    let
        lines = String.lines formatted
        isHeading l = String.startsWith "#" (String.trim l)
        countLeadingHashes s =
            case String.uncons s of
                Just ( '#', rest ) -> 1 + countLeadingHashes rest
                _ -> 0
        stripHashes l =
            let
                trimmed = String.trim l
            in
            String.dropLeft (countLeadingHashes trimmed) trimmed |> String.trim
        firstHeading =
            lines
                |> List.filter isHeading
                |> List.head
                |> Maybe.map stripHashes
                |> Maybe.withDefault
                    (lines
                        |> List.filter (\l -> not (String.isEmpty (String.trim l)))
                        |> List.head
                        |> Maybe.withDefault "Voice ticket"
                    )
        firstHeadingLine =
            lines
                |> List.filter isHeading
                |> List.head
        description =
            case firstHeadingLine of
                Just headingLine ->
                    let
                        dropFirst remaining =
                            case remaining of
                                [] -> []
                                x :: xs ->
                                    if x == headingLine then xs
                                    else x :: dropFirst xs
                    in
                    dropFirst lines
                        |> String.join "\n"
                        |> String.trim

                Nothing ->
                    lines
                        |> String.join "\n"
                        |> String.trim
    in
    ( firstHeading, description )


stopRecordingIfActive : VoiceState -> Cmd msg
stopRecordingIfActive vs =
    case vs.recordingState of
        VoiceRecording ->
            stopRecording True

        _ ->
            Cmd.none


-- SUBSCRIPTIONS

subscriptions : Model -> Sub Msg
subscriptions model =
    let
        telegramPoll =
            case model.chatState.activeChatId of
                Just cid ->
                    if isTelegramConversation cid model.chatState then
                        Time.every 3000 (\_ -> ChatPollTelegram cid)
                    else
                        Sub.none

                Nothing ->
                    Sub.none
    in
    Sub.batch
        [ Time.every 2000 Tick
        , audioRecorded
            (\val ->
                case D.decodeValue decodeAudioRecorded val of
                    Ok rec ->
                        VoiceGotAudio rec.data rec.mimeType

                    Err _ ->
                        VoiceRecordingFailed "Failed to decode audio recording data"
            )
        , audioRecordingError VoiceRecordingFailed
        , chatWsMessage ChatWsReceived
        , fileSelected ChatFileReceived
        , videoRecorded ChatVideoReceived
        , telegramPoll
        ]


decodeAudioRecorded : D.Decoder { data : String, mimeType : String }
decodeAudioRecorded =
    D.map2 (\d m -> { data = d, mimeType = m })
        (D.field "data" D.string)
        (D.field "mimeType" D.string)


extensionForMime : String -> String
extensionForMime mime =
    if String.startsWith "audio/ogg" mime then
        ".ogg"

    else if String.startsWith "audio/webm" mime then
        ".webm"

    else if String.startsWith "audio/mpeg" mime then
        ".mp3"

    else
        ".webm"


-- VIEW

view : Model -> Browser.Document Msg
view model =
    { title = "Twolebot"
    , body =
        [ UI.appShell
            [ case model.route of
                SetupRoute ->
                    -- No header on setup page
                    text ""

                _ ->
                    UI.header model.backendOnline model.route Navigate
            , viewSetupBanner model
            , main_
                [ style "padding-bottom" "2rem" ]
                [ viewWorkError model.workError
                , viewWorkNotice model.workNotice
                , case model.route of
                    SetupRoute ->
                        viewSetupPage model

                    _ ->
                        UI.container [ viewPage model ]
                ]
            , viewKeyframes
            ]
        ]
    }


viewSetupBanner : Model -> Html Msg
viewSetupBanner model =
    case ( model.route, model.setupStatus ) of
        ( SetupRoute, _ ) ->
            text ""

        ( _, Success status ) ->
            if not status.isComplete then
                div
                    [ style "background" ("linear-gradient(90deg, " ++ UI.colors.warningDim ++ ", " ++ UI.colors.bgSecondary ++ ")")
                    , style "border-bottom" ("1px solid " ++ UI.colors.warning)
                    , style "padding" "0.6rem 1rem"
                    , style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    , style "cursor" "pointer"
                    , onClick (Navigate SetupRoute)
                    ]
                    [ span [ style "font-size" "1.1rem" ] [ text "\u{1F9D9}" ]
                    , span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.75rem"
                        , style "color" UI.colors.warning
                        , style "flex" "1"
                        ]
                        [ text "Setup incomplete — continue the setup wizard" ]
                    , span
                        [ style "color" UI.colors.warning
                        , style "font-size" "0.75rem"
                        ]
                        [ text "\u{203A}" ]
                    ]
            else
                text ""

        _ ->
            text ""


viewWorkError : Maybe String -> Html Msg
viewWorkError maybeError =
    case maybeError of
        Nothing ->
            text ""

        Just err ->
            div
                [ style "position" "fixed"
                , style "top" "72px"
                , style "left" "50%"
                , style "transform" "translateX(-50%)"
                , style "z-index" "150"
                , style "max-width" "600px"
                , style "width" "90%"
                ]
                [ div
                    [ style "background-color" UI.colors.errorDim
                    , style "border" ("1px solid " ++ UI.colors.error)
                    , style "border-radius" "4px"
                    , style "padding" "0.75rem 1rem"
                    , style "display" "flex"
                    , style "justify-content" "space-between"
                    , style "align-items" "center"
                    , style "gap" "0.75rem"
                    ]
                    [ span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.75rem"
                        , style "color" UI.colors.error
                        ]
                        [ text err ]
                    , button
                        [ onClick ClearWorkError
                        , style "background" "transparent"
                        , style "border" "none"
                        , style "color" UI.colors.error
                        , style "cursor" "pointer"
                        , style "font-size" "1rem"
                        , style "padding" "0.25rem"
                        , style "line-height" "1"
                        ]
                        [ text "x" ]
                    ]
                ]


viewWorkNotice : Maybe String -> Html Msg
viewWorkNotice maybeNotice =
    case maybeNotice of
        Nothing ->
            text ""

        Just noticeText ->
            div
                [ style "position" "fixed"
                , style "top" "124px"
                , style "left" "50%"
                , style "transform" "translateX(-50%)"
                , style "z-index" "149"
                , style "max-width" "600px"
                , style "width" "90%"
                ]
                [ div
                    [ style "background-color" UI.colors.successDim
                    , style "border" ("1px solid " ++ UI.colors.success)
                    , style "border-radius" "4px"
                    , style "padding" "0.75rem 1rem"
                    , style "display" "flex"
                    , style "justify-content" "space-between"
                    , style "align-items" "center"
                    , style "gap" "0.75rem"
                    ]
                    [ span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.75rem"
                        , style "color" UI.colors.success
                        ]
                        [ text noticeText ]
                    , button
                        [ onClick ClearWorkNotice
                        , style "background" "none"
                        , style "border" "none"
                        , style "color" UI.colors.success
                        , style "cursor" "pointer"
                        , style "font-size" "0.875rem"
                        ]
                        [ text "×" ]
                    ]
                ]


viewSetupPage : Model -> Html Msg
viewSetupPage model =
    case model.setupStatus of
        NotAsked ->
            div [ class "loading" ] [ text "Loading..." ]

        Loading ->
            div [ class "loading" ] [ text "Loading..." ]

        Failure err ->
            div [ class "error" ] [ text ("Error: " ++ err) ]

        Success status ->
            Pages.Setup.view
                status
                model.telegramTokenInput
                model.geminiKeyInput
                model.allowedUsernameSetupInput
                { telegramInput = TelegramTokenInput
                , geminiInput = GeminiKeyInput
                , usernameInput = AllowedUsernameSetupInput
                , submitTelegram = SubmitTelegramToken
                , submitGemini = SubmitGeminiKey
                , submitUsername = SubmitAllowedUsername
                , installClaude = InstallClaude
                , checkClaudeAuth = CheckClaudeAuth
                , updateClaude = UpdateClaude
                , testClaude = TestClaude
                , checkThreading = CheckThreading
                , goToDashboard = Navigate DashboardRoute
                }


viewPage : Model -> Html Msg
viewPage model =
    case model.route of
        DashboardRoute ->
            Pages.Dashboard.view model.feedData model.responsesData model.semanticStatus model.tunnelStatus model.currentTime Refresh ToggleSemanticIndexer TriggerSemanticReindex

        MessagesRoute selectedChat topicFilter ->
            Pages.Messages.view
                selectedChat
                topicFilter
                model.chats
                model.messagesPage
                model.messageSearch
                BackToChats
                MessagesPageChange
                MessageSearchChange
                MessageSearchSubmit
                Refresh

        LogsRoute ->
            Pages.Logs.view
                model.logsPage
                model.logSearch
                LogSearchChange
                LogSearchSubmit
                LogsPageChange
                Refresh

        SettingsRoute ->
            let
                hasUserContacted = model.apiKeys |> Maybe.andThen .hasUserContacted
                botName = model.apiKeys
                    |> Maybe.andThen .telegramStatus
                    |> Maybe.andThen .info
                    |> Maybe.andThen (\info ->
                        -- info is like "@botname" or "@botname (group access)"
                        if String.startsWith "@" info then
                            info |> String.dropLeft 1 |> String.split " " |> List.head
                        else
                            Nothing
                    )
            in
            Pages.Settings.view
                model.settings
                model.settingsSaving
                ToggleToolMessages
                ToggleThinkingMessages
                ToggleToolResults
                ChangeChatHarness
                ChangeClaudeModel
                SaveClaudeModel
                ChangeDevRolePrompt
                SaveDevRolePrompt
                ChangeHardenRolePrompt
                SaveHardenRolePrompt
                ChangePmRolePrompt
                SavePmRolePrompt
                model.apiKeys
                model.apiKeysSaving
                model.apiKeysError
                model.telegramTokenEdit
                model.geminiKeyEdit
                TelegramTokenEditChange
                GeminiKeyEditChange
                SaveApiKeys
                Refresh
                model.allowedUsernameInput
                AllowedUsernameInputChange
                SaveAllowedUsername
                ClearAllowedUsername
                hasUserContacted
                botName

        CronJobsRoute ->
            Pages.CronJobs.view
                model.cronJobs
                model.cronStatus
                Refresh
                PauseCronJob
                ResumeCronJob
                CancelCronJob

        CapabilitiesRoute ->
            Pages.Capabilities.view

        WelcomeRoute ->
            Pages.Welcome.view

        SetupRoute ->
            -- Handled in viewSetupPage
            text ""

        ProjectsRoute ->
            Pages.Projects.view
                model.workProjects
                model.showProjectForm
                model.projectForm
                model.workBusy
                Navigate
                Refresh
                ProjectNameChange
                ProjectDescChange
                ProjectTagsChange
                ProjectGitRemoteUrlChange
                ToggleProjectForm
                SubmitProject
                CloseProjectForm

        ProjectDetailRoute _ ->
            Pages.ProjectDetail.view
                model.workProject
                model.workTasks
                model.workDocuments
                model.workActivity
                model.workAnalytics
                model.projectTab
                model.showTaskForm
                model.taskForm
                model.showDocumentForm
                model.documentForm
                model.taskFilters
                model.taskViewMode
                model.draggingTaskId
                model.boardDropTarget
                model.boardDropComment
                model.workBusy
                model.voiceState
                { onNavigate = Navigate
                , onRefresh = Refresh
                , onTabChange = ChangeProjectTab
                , onToggleTaskForm = ToggleTaskForm
                , onTaskTitleChange = TaskTitleChange
                , onTaskDescChange = TaskDescChange
                , onTaskPriorityChange = TaskPriorityChange
                , onSubmitTask = SubmitTask
                , onCloseTaskForm = CloseTaskForm
                , onToggleDocForm = ToggleDocForm
                , onDocTitleChange = DocTitleChange
                , onDocContentChange = DocContentChange
                , onDocTypeChange = DocTypeChange
                , onSubmitDoc = SubmitDocument
                , onCloseDocForm = CloseDocForm
                , onToggleStatus = ToggleStatusFilter
                , onClearFilters = ClearFilters
                , onTakeNextTask = TakeNextTaskAction
                , onTakeNextReviewTask = TakeNextReviewTaskAction
                , onToggleViewMode = ToggleTaskViewMode
                , onDragStart = DragStart
                , onDragEnd = DragEnd
                , onDragOver = DragOver
                , onDropOnStatus = DropOnStatus
                , onBoardDropCommentChange = BoardDropCommentChange
                , onSubmitBoardDrop = SubmitBoardDrop
                , onCancelBoardDrop = CancelBoardDrop
                , onVoiceStartRecording = VoiceStartRecording
                , onVoiceStopRecording = VoiceStopRecording
                , onVoiceReset = VoiceReset
                , onVoiceApplyToForm = VoiceApplyToTaskForm
                }

        TaskDetailRoute _ ->
            Pages.TaskDetail.view
                model.workTask
                model.workComments
                model.commentForm
                model.replyingToCommentId
                model.editingCommentId
                model.collapsedComments
                model.rejectReviewComment
                model.workBusy
                model.pendingStatusChange
                model.readyForReviewComment
                model.editingField
                model.voiceState
                { onNavigate = Navigate
                , onRefresh = Refresh
                , onStatusChange = ChangeTaskStatus
                , onMoveTop = MoveTaskToTop
                , onMoveBottom = MoveTaskToBottom
                , onCommentChange = CommentContentChange
                , onSubmitComment = SubmitComment
                , onSubmitReply = SubmitReply
                , onStartReply = StartReply
                , onCancelReply = CancelReply
                , onStartEdit = StartEditComment
                , onSaveEdit = SaveEditedComment
                , onCancelEdit = CancelEditComment
                , onToggleCommentCollapse = ToggleCommentCollapse
                , onRejectReviewCommentChange = RejectReviewCommentChange
                , onRejectReview = RejectReviewAction
                , onReadyForReviewCommentChange = ReadyForReviewCommentChange
                , onSubmitReadyForReview = SubmitReadyForReview
                , onCancelPendingStatus = CancelPendingStatus
                , onStartEditField = StartEditField
                , onCancelEditField = CancelEditField
                , onSaveEditField = SaveEditField
                , onChangePriority = ChangeTaskPriority
                , onVoiceStartRecording = VoiceStartRecording
                , onVoiceStopRecording = VoiceStopRecording
                , onVoiceReset = VoiceReset
                , onVoiceApplyEdit = VoiceApplyToTaskEdit
                , onVoiceApplyToComment = VoiceApplyToComment
                , onVoiceEditComment = VoiceEditComment
                }

        DocumentDetailRoute _ ->
            Pages.DocumentDetail.view
                model.workDocument
                model.workComments
                model.commentForm
                model.replyingToCommentId
                model.editingCommentId
                model.collapsedComments
                model.workBusy
                model.voiceState
                { onNavigate = Navigate
                , onRefresh = Refresh
                , onCommentChange = CommentContentChange
                , onSubmitComment = SubmitComment
                , onSubmitReply = SubmitReply
                , onStartReply = StartReply
                , onCancelReply = CancelReply
                , onStartEdit = StartEditComment
                , onSaveEdit = SaveEditedComment
                , onCancelEdit = CancelEditComment
                , onToggleCommentCollapse = ToggleCommentCollapse
                , onVoiceStartRecording = VoiceStartRecording
                , onVoiceStopRecording = VoiceStopRecording
                , onVoiceReset = VoiceReset
                , onVoiceApplyToComment = VoiceApplyToComment
                , onVoiceEditComment = VoiceEditComment
                }

        LiveBoardRoute ->
            Pages.LiveBoard.view
                model.workLiveBoard
                model.workProjects
                model.workBusy
                model.taskViewMode
                model.draggingTaskId
                model.boardDropTarget
                model.boardDropComment
                { onNavigate = Navigate
                , onRefresh = Refresh
                , onSelectTask = SelectLiveBoardTask
                , onDeselectTask = DeselectLiveBoardTask
                , onClearCompleted = ClearCompletedTasks
                , onMoveSelection = MoveLiveSelection
                , onEnsureAgent = EnsureAgentLoopAction
                , onStopAgent = StopAgentLoopAction
                , onToggleViewMode = ToggleTaskViewMode
                , onDragStart = DragStart
                , onDragEnd = DragEnd
                , onDragOver = DragOver
                , onDropOnStatus = DropOnStatus
                , onBoardDropCommentChange = BoardDropCommentChange
                , onSubmitBoardDrop = SubmitBoardDrop
                , onCancelBoardDrop = CancelBoardDrop
                }

        ChatRoute _ ->
            Pages.Chat.view
                model.chatState
                { onNavigate = Navigate
                , onNewConversation = ChatNewConversation
                , onSelectConversation = ChatSelectConversation
                , onInputChange = ChatInputChange
                , onSendMessage = ChatSendMessage
                , onStartRename = ChatStartRename
                , onRenameChange = ChatRenameChange
                , onSubmitRename = ChatSubmitRename
                , onCancelRename = ChatCancelRename
                , onStartVoice = ChatStartVoice
                , onStopVoice = ChatStopVoice
                , onCancelVoice = ChatCancelVoice
                , onAttachFile = ChatAttachFile
                , onStartVideo = ChatStartVideo
                , onStopVideo = ChatStopVideo
                , onCancelVideo = ChatCancelVideo
                , onConfirmDelete = ChatConfirmDelete
                , onCancelDelete = ChatCancelDelete
                , onDeleteConversation = ChatDeleteConversation
                , onDismissError = ChatDismissError
                , onDismissNotification = ChatDismissNotification
                , onRetryUpload = ChatRetryUpload
                }


viewKeyframes : Html msg
viewKeyframes =
    node "style" []
        [ text """
            /* Import IBM Plex fonts for industrial aesthetic */
            @import url('https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600;700&family=IBM+Plex+Sans+Condensed:wght@400;500;600;700&family=IBM+Plex+Sans:wght@400;500;600&family=JetBrains+Mono:wght@400;500;600;700&display=swap');

            @keyframes spin {
                from { transform: rotate(0deg); }
                to { transform: rotate(360deg); }
            }

            @keyframes statusPulse {
                0%, 100% { opacity: 1; box-shadow: 0 0 8px currentColor; }
                50% { opacity: 0.6; box-shadow: 0 0 16px currentColor; }
            }

            @keyframes pulse {
                0%, 100% { opacity: 1; transform: scale(1); }
                50% { opacity: 0.7; transform: scale(0.97); }
            }

            @keyframes glow {
                0%, 100% { box-shadow: 0 0 8px rgba(0, 212, 170, 0.4); }
                50% { box-shadow: 0 0 20px rgba(0, 212, 170, 0.6); }
            }

            * { box-sizing: border-box; margin: 0; padding: 0; }

            body {
                line-height: 1.6;
                background-color: #0a0e14;
            }

            /* Button interactions */
            button { transition: all 0.15s ease; }
            button:hover {
                filter: brightness(1.1);
                transform: translateY(-1px);
            }
            button:active { transform: translateY(0); }

            /* Scrollbar - industrial minimal */
            ::-webkit-scrollbar { width: 6px; height: 6px; }
            ::-webkit-scrollbar-track { background: #0a0e14; }
            ::-webkit-scrollbar-thumb {
                background: #21262d;
                border-radius: 0;
            }
            ::-webkit-scrollbar-thumb:hover { background: #30363d; }

            /* Mobile tabs: keep nav intentional on small screens */
            .tb-mobile-tabs { display: none; }
            .tb-mobile-tabs__inner::-webkit-scrollbar { display: none; }
            .tb-mobile-tabs__inner { scrollbar-width: none; }

            @media (max-width: 720px) {
                .tb-desktop-nav { display: none !important; }
                .tb-mobile-tabs { display: block !important; }
                .tb-status-chip { display: none !important; }
                .tb-version-shell { display: none !important; }
                .tb-brand-shell {
                    border-right: none !important;
                    padding-right: 0 !important;
                }

                /* Chat: mobile layout — show sidebar OR main, not both */
                .chat-container {
                    border: none !important;
                    border-radius: 0 !important;
                    margin-top: 0 !important;
                    height: calc(100vh - 110px) !important;
                }
                /* When a conversation is open: go fullscreen overlay */
                .chat-container--has-active {
                    position: fixed !important;
                    top: 0 !important;
                    left: 0 !important;
                    right: 0 !important;
                    bottom: 0 !important;
                    height: 100vh !important;
                    height: 100dvh !important;
                    z-index: 110 !important;
                    margin: 0 !important;
                }
                .chat-sidebar {
                    width: 100% !important;
                    min-width: 100% !important;
                    border-right: none !important;
                }
                .chat-sidebar--has-active {
                    display: none !important;
                }
                .chat-main {
                    display: none !important;
                }
                .chat-main--active {
                    display: flex !important;
                    width: 100% !important;
                }
                .chat-back-header {
                    display: flex !important;
                }
            }

            /* Hide keyboard shortcut hint on mobile/touch devices */
            @media (max-width: 720px) {
                .desktop-shortcut-hint { display: none !important; }
            }
            @media (hover: none) {
                .desktop-shortcut-hint { display: none !important; }
            }

            /* Conversation delete button: show on hover (desktop) */
            .conv-item:hover .conv-delete-btn {
                opacity: 1 !important;
            }
            .conv-delete-btn:hover {
                background: rgba(248, 81, 73, 0.15) !important;
                color: #f85149 !important;
            }
            @media (max-width: 720px) {
                .conv-delete-btn { opacity: 0.6 !important; }
            }
            @media (hover: none) {
                .conv-delete-btn { opacity: 0.6 !important; }
            }

            /* Media elements */
            audio, video {
                max-width: 100%;
                border-radius: 2px;
            }

            /* Selection styling */
            ::selection {
                background: rgba(0, 212, 170, 0.3);
                color: #e6edf3;
            }

            /* Focus states */
            button:focus-visible, input:focus-visible {
                outline: 1px solid #00d4aa;
                outline-offset: 2px;
            }

            .loading {
                text-align: center;
                padding: 4rem 2rem;
                color: #8b949e;
                font-family: 'IBM Plex Sans', sans-serif;
            }

            .error {
                text-align: center;
                padding: 2rem;
                color: #f85149;
                font-family: 'IBM Plex Sans', sans-serif;
            }
        """
        ]
