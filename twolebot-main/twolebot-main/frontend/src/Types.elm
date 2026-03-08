module Types exposing (..)

import Browser.Navigation as Nav
import Dict exposing (Dict)
import Time


-- Prompts

type alias PromptItem =
    { id : String
    , sourceType : String
    , userId : Int
    , prompt : String
    , mediaPath : Maybe String
    , status : String
    , createdAt : String
    , startedAt : Maybe String
    , completedAt : Maybe String
    , error : Maybe String
    }


type alias FeedData =
    { pending : List PromptItem
    , pendingCount : Int
    , running : Maybe PromptItem
    , recentCompleted : List PromptItem
    , completedCount : Int
    }


-- Responses

type alias ResponseItem =
    { id : String
    , promptId : String
    , sourceType : String
    , userId : Int
    , content : String
    , isPartial : Bool
    , isFinal : Bool
    , sequence : Int
    , status : String
    , createdAt : String
    , sentAt : Maybe String
    , nextAttemptAt : Maybe String
    , error : Maybe String
    }


type alias ResponseFeedData =
    { pending : List ResponseItem
    , pendingCount : Int
    , recentSent : List ResponseItem
    , sentCount : Int
    , recentFailed : List ResponseItem
    , failedCount : Int
    }


-- Messages

type alias StoredMessage =
    { id : String
    , chatId : String
    , userId : Maybe Int
    , direction : String
    , content : String
    , mediaType : Maybe String
    , mediaPath : Maybe String
    , timestamp : String
    }


type alias ChatSummary =
    { chatId : String
    , topicId : Maybe Int
    , username : Maybe String
    , displayName : Maybe String
    , messageCount : Int
    }


type alias MessagesPage =
    { messages : List StoredMessage
    , total : Int
    , page : Int
    , pageSize : Int
    , totalPages : Int
    }


-- Logs

type alias LogEntry =
    { timestamp : String
    , level : String
    , component : String
    , message : String
    }


type alias LogsPage =
    { entries : List LogEntry
    , total : Int
    , page : Int
    , pageSize : Int
    , totalPages : Int
    }


-- Settings

type alias Settings =
    { showToolMessages : Bool
    , showThinkingMessages : Bool
    , showToolResults : Bool
    , ompNumThreads : Int
    , allowedUsername : Maybe String
    , chatHarness : String
    , claudeModel : String
    , devRolePrompt : String
    , hardenRolePrompt : String
    , pmRolePrompt : String
    }


defaultSettings : Settings
defaultSettings =
    { showToolMessages = False
    , showThinkingMessages = False
    , showToolResults = False
    , ompNumThreads = 2
    , allowedUsername = Nothing
    , chatHarness = "claude"
    , claudeModel = "claude-opus-4-6"
    , devRolePrompt = ""
    , hardenRolePrompt = ""
    , pmRolePrompt = ""
    }


-- API Keys

type alias ApiKeyStatus =
    { valid : Bool
    , error : Maybe String
    , info : Maybe String
    }


type alias ApiKeysData =
    { hasTelegramToken : Bool
    , telegramTokenMasked : Maybe String
    , telegramStatus : Maybe ApiKeyStatus
    , hasGeminiKey : Bool
    , geminiKeyMasked : Maybe String
    , geminiStatus : Maybe ApiKeyStatus
    , claudeCodeStatus : Maybe ClaudeCodeStatus
    , hasUserContacted : Maybe Bool
    }


type alias ClaudeCodeStatus =
    { authMode : String
    , accountEmail : Maybe String
    , accountName : Maybe String
    , organization : Maybe String
    }


-- Cron Jobs

type alias CronJob =
    { id : String
    , name : Maybe String
    , schedule : String
    , status : String  -- "active", "paused", "cancelled"
    , deferrable : Bool
    , nextRun : Maybe String
    , lastRun : Maybe String
    , createdAt : String
    }


type alias CronStatus =
    { activeJobs : Int
    , pausedJobs : Int
    , waitingExecutions : Int
    }


-- Semantic Indexer

type alias SemanticStatus =
    { enabled : Bool
    , memory : TaskStatus
    , conversations : TaskStatus
    , totalMemoryChunks : Int
    , totalMemoryFiles : Int
    , totalConversationChunks : Int
    , totalConversationSessions : Int
    , totalMemoryFilesAvailable : Int
    , totalConversationFilesAvailable : Int
    , memoryFilesStale : Int
    , conversationFilesStale : Int
    , lastConversationPollAt : Maybe Int
    , conversationPollIntervalSecs : Int
    }


type alias TaskStatus =
    { activity : String  -- "idle", "initial_index", "indexing", "polling"
    , currentFile : Maybe String
    , filesIndexed : Int
    , filesSkipped : Int
    , filesTotal : Maybe Int
    , chunksProcessed : Int
    , chunksTotal : Maybe Int
    }


-- Tunnel

type alias TunnelStatus =
    { active : Bool
    , url : Maybe String
    , qrSvg : Maybe String
    }


-- Routes

type Route
    = DashboardRoute
    | MessagesRoute (Maybe String) (Maybe String)  -- chatId, topicFilter
    | LogsRoute
    | CronJobsRoute
    | SettingsRoute
    | CapabilitiesRoute
    | WelcomeRoute
    | SetupRoute
    -- Work routes
    | ProjectsRoute
    | ProjectDetailRoute Int
    | TaskDetailRoute Int
    | DocumentDetailRoute Int
    | LiveBoardRoute
    | ChatRoute (Maybe String)  -- active conversation id


-- Setup


type alias SetupMsgs msg =
    { telegramInput : String -> msg
    , geminiInput : String -> msg
    , usernameInput : String -> msg
    , submitTelegram : msg
    , submitGemini : msg
    , submitUsername : msg
    , installClaude : msg
    , checkClaudeAuth : msg
    , updateClaude : msg
    , testClaude : msg
    , checkThreading : msg
    , goToDashboard : msg
    }


type alias SetupStatus =
    { dataDir : String
    , hasTelegramToken : Bool
    , hasGeminiKey : Bool
    , hasClaudeCli : Bool
    , claudeCliVersion : Maybe String
    , hasAllowedUsername : Bool
    , isComplete : Bool
    , platform : String
    , botName : Maybe String
    , telegramError : Maybe String
    , geminiError : Maybe String
    , claudeInstalling : Bool
    , claudeInstallError : Maybe String
    , allowedUsernameError : Maybe String
    -- Claude auth/update state
    , claudeAuthenticated : Bool
    , claudeAuthMode : Maybe String
    , claudeAccountEmail : Maybe String
    , claudeAccountName : Maybe String
    , claudeNeedsUpdate : Bool
    , claudeLatestVersion : Maybe String
    , claudeUpdating : Bool
    , claudeUpdateError : Maybe String
    , claudeTesting : Bool
    , claudeTestResult : Maybe Bool
    , claudeTestOutput : Maybe String
    , claudeTestError : Maybe String
    , claudeAuthChecking : Bool
    -- Threading state
    , hasThreadingEnabled : Bool
    , threadingChecking : Bool
    , threadingError : Maybe String
    -- Previews from server
    , geminiKeyPreview : Maybe String
    , allowedUsernameValue : Maybe String
    }


-- App State

type alias Model =
    { navKey : Nav.Key
    , route : Route
    , backendOnline : Bool
    , feedData : RemoteData FeedData
    , responsesData : RemoteData ResponseFeedData
    , chats : RemoteData (List ChatSummary)
    , messagesPage : RemoteData MessagesPage
    , messageSearch : String
    , messagesPageIndex : Int
    , logsPage : RemoteData LogsPage
    , logSearch : String
    , logsPageIndex : Int
    , currentTime : Time.Posix
    -- Settings state
    , settings : RemoteData Settings
    , settingsSaving : Bool
    -- API Keys state (in settings)
    , apiKeys : Maybe ApiKeysData
    , apiKeysLoading : Bool
    , apiKeysSaving : Bool
    , apiKeysError : Maybe String
    , telegramTokenEdit : String
    , geminiKeyEdit : String
    , allowedUsernameInput : String
    -- Cron state
    , cronJobs : RemoteData (List CronJob)
    , cronStatus : RemoteData CronStatus
    -- Semantic indexer state
    , semanticStatus : RemoteData SemanticStatus
    -- Tunnel state
    , tunnelStatus : RemoteData TunnelStatus
    -- Setup state
    , setupStatus : RemoteData SetupStatus
    , telegramTokenInput : String
    , geminiKeyInput : String
    , allowedUsernameSetupInput : String
    -- Work state
    , workProjects : RemoteData (List WorkProject)
    , workProject : RemoteData WorkProject
    , workTasks : RemoteData (List WorkTask)
    , workTask : RemoteData WorkTask
    , workDocuments : RemoteData (List WorkDocument)
    , workDocument : RemoteData WorkDocument
    , workComments : RemoteData (List WorkComment)
    , workActivity : RemoteData (List ActivityLog)
    , workAnalytics : RemoteData TaskAnalytics
    , workLiveBoard : RemoteData LiveBoard
    -- Work UI state
    , projectTab : ProjectTab
    , projectForm : ProjectForm
    , showProjectForm : Bool
    , taskForm : TaskForm
    , showTaskForm : Bool
    , documentForm : DocumentForm
    , showDocumentForm : Bool
    , commentForm : CommentForm
    , replyingToCommentId : Maybe Int
    , editingCommentId : Maybe Int
    , collapsedComments : List Int
    , taskFilters : TaskFilters
    , taskViewMode : TaskViewMode
    , draggingTaskId : Maybe Int
    , boardDropComment : String
    , boardDropTarget : Maybe ( Int, String )
    , editingField : EditingField
    , rejectReviewComment : String
    , pendingStatusChange : Maybe String
    , readyForReviewComment : String
    , workBusy : Bool
    , workError : Maybe String
    , workNotice : Maybe String
    -- Voice state
    , voiceState : VoiceState
    -- Chat state
    , chatState : ChatPageState
    }


type RemoteData a
    = NotAsked
    | Loading
    | Success a
    | Failure String


-- Helpers

isLoading : RemoteData a -> Bool
isLoading rd =
    case rd of
        Loading -> True
        _ -> False


withDefault : a -> RemoteData a -> a
withDefault default rd =
    case rd of
        Success a -> a
        _ -> default


-- ═══════════════════════════════════════════════════════════════════════════
-- CHAT TYPES
-- ═══════════════════════════════════════════════════════════════════════════


type alias Conversation =
    { id : String
    , name : String
    , customName : Maybe String
    , autoName : Maybe String
    , displayName : Maybe String
    , protocol : Maybe String
    , lastMessagePreview : Maybe String
    , updatedAt : String
    }


type MessageDirection
    = Inbound
    | Outbound


type alias ChatMessage =
    { id : String
    , direction : MessageDirection
    , content : String
    , timestamp : String
    , attachments : List MediaAttachment
    }


type MediaAttachment
    = AudioAttachment { path : String, transcription : Maybe String }
    | VideoAttachment { path : String, transcription : Maybe String }
    | ImageAttachment { path : String, description : Maybe String }
    | FileAttachment { path : String, name : String, mimeType : String }


mediaAttachmentFromServerFields : Maybe String -> Maybe String -> List MediaAttachment
mediaAttachmentFromServerFields maybeType maybePath =
    case ( maybeType, maybePath ) of
        ( Just mediaType, Just path ) ->
            if String.startsWith "audio" mediaType || mediaType == "voice" then
                [ AudioAttachment { path = path, transcription = Nothing } ]

            else if String.startsWith "video" mediaType || mediaType == "video" then
                [ VideoAttachment { path = path, transcription = Nothing } ]

            else if String.startsWith "image" mediaType || mediaType == "photo" then
                [ ImageAttachment { path = path, description = Nothing } ]

            else
                [ FileAttachment { path = path, name = path, mimeType = mediaType } ]

        _ ->
            []


-- ── Chat Activity State Machine ──────────────────────────────────────────
--
-- Exactly ONE of these is active at any time per conversation. The compiler
-- enforces that you cannot be simultaneously sending a message, streaming
-- a response, and recording audio.
--
-- NAVIGATION RULE: ComposingVoice and ComposingVideo LOCK the UI to this
-- conversation (hardware is active). The user must stop or cancel before
-- switching. All other states allow free navigation — uploads, streaming,
-- and awaiting responses continue in the background via the Dict.
--
-- UPLOAD RULE: Media uploads (voice/video/file) run on the upload queue
-- (List UploadTask), NOT on ChatActivity. This means you can drop 3 files,
-- record a voice note, and keep typing — uploads proceed independently.
-- Only text sending uses ChatActivity.ChatSending, because it blocks
-- further text input until the send completes.
--
-- Valid ChatActivity transitions:
--
--   Idle ──► Composing Text         (user types in input)
--   Idle ──► Composing Voice        (user taps record audio)
--   Idle ──► Composing Video        (user taps record video)
--
--   Composing Text ──► Sending Text        (user hits send)
--   Composing Text ──► Idle                (user clears input)
--   Composing Text ──► Composing Voice     (user taps record while text present)
--   Composing Text ──► Composing Video     (user taps video while text present)
--
--   Composing Voice ──► Idle + enqueue upload   (user stops recording)
--   Composing Voice ──► Idle                    (user cancels recording)
--     ⚠ BLOCKS navigation until stopped or cancelled
--
--   Composing Video ──► Idle + enqueue upload   (user stops recording)
--   Composing Video ──► Idle                    (user cancels recording)
--     ⚠ BLOCKS navigation until stopped or cancelled
--
--   Sending Text ──► AwaitingResponse     (HTTP 200)
--   Sending Text ──► Error SendFailed     (HTTP error)
--     ✓ User can navigate away
--
--   Transcribing ──► AwaitingResponse       (typing_indicator WS after transcription)
--   Transcribing ──► Streaming             (first WS chunk arrives)
--
--   AwaitingResponse ──► Streaming         (first WS chunk / typing indicator)
--   AwaitingResponse ──► Error Timeout     (no WS event within timeout)
--     ✓ User can navigate away
--
--   Streaming ──► Idle                     (is_final = true)
--   Streaming ──► Error StreamInterrupted  (WS connection lost mid-stream)
--     ✓ User can navigate away
--
--   Error * ──► Idle                       (user dismisses)
--   Error * ──► Sending Text              (user retries text send)
--
--   Idle ──► Observing                    (navigate to telegram conversation)
--   Observing ──► Idle                    (navigate away from telegram conversation)
--
-- Upload queue transitions (independent, per-task):
--
--   File selected / recording stopped ──► Uploading
--   Uploading ──► UploadSucceeded   (HTTP 200 from /api/chat/upload)
--   Uploading ──► UploadFailed      (HTTP error)
--   UploadFailed ──► Uploading      (user retries)

type ChatActivity
    = ChatIdle
    | ChatComposing ComposingMode
    | ChatSending SendingPayload
    | ChatTranscribing
    | ChatAwaitingResponse { pendingId : String }
    | ChatStreaming { buffer : String }
    | ChatError ChatErrorInfo
    | ChatObserving


type ComposingMode
    = ComposingText
    | ComposingVoice
    | ComposingVideo


type SendingPayload
    = SendingText { content : String, pendingId : String }


-- ── Upload Queue ─────────────────────────────────────────────────────────
--
-- Uploads run independently of ChatActivity. The user can drop 3 files,
-- start recording voice, or keep typing — uploads proceed in the
-- background. Each upload becomes its own message on the server (the
-- backend's POST /api/chat/upload is atomic: store + transcribe + enqueue).
--
-- The queue lives on ChatConversationState so uploads continue even when
-- the user switches to another conversation.

type alias UploadTask =
    { id : String
    , media : MediaUploadPayload
    , status : UploadStatus
    }


type MediaUploadPayload
    = UploadVoice { data : String, mimeType : String }
    | UploadVideo { data : String, mimeType : String }
    | UploadFile { data : String, name : String, mimeType : String }


type UploadStatus
    = Uploading
    | UploadSucceeded { transcription : Maybe String, mediaPath : String }
    | UploadFailed String


type alias ChatErrorInfo =
    { error : ChatError
    , retryable : Bool
    , failedContent : Maybe String
    }


type ChatError
    = SendFailed String
    | StreamInterrupted String
    | ConnectionLost
    | MediaUploadFailed String


-- ── WebSocket Connection State Machine ───────────────────────────────────
--
--   Disconnected ──► Connected            (JS sends "connected" after WS open + server frame)
--   Connected ──► Disconnected            (navigate away / explicit close)
--   Connected ──► Reconnecting            (WebSocket.onclose, resumes from last_seq)
--   Reconnecting ──► Connected            (reconnect succeeds)
--   Reconnecting ──► Disconnected         (max retries / navigate away)

type WsConnection
    = WsDisconnected
    | WsConnected String
    | WsReconnecting String Int


-- ── Per-Conversation State ────────────────────────────────────────────────
--
-- Each conversation gets its own independent state machine instance.
-- Background conversations keep running (uploads, WS streams) when
-- you switch away — their state lives here in the Dict, not on the
-- "active" view.

type alias ChatConversationState =
    { messages : List ChatMessage
    , pendingOutbound : List ChatMessage
    , uploads : List UploadTask
    , activity : ChatActivity
    , connection : WsConnection
    , inputText : String
    , messageCounter : Int
    , uploadCounter : Int
    , messagesLoaded : MessageLoadState
    , lastChunkSequence : Int
    }


type MessageLoadState
    = MessagesNotLoaded
    | MessagesLoading
    | MessagesLoaded { hasMore : Bool }
    | MessagesLoadError String


emptyChatConversationState : ChatConversationState
emptyChatConversationState =
    { messages = []
    , pendingOutbound = []
    , uploads = []
    , activity = ChatIdle
    , connection = WsDisconnected
    , inputText = ""
    , messageCounter = 0
    , uploadCounter = 0
    , messagesLoaded = MessagesNotLoaded
    , lastChunkSequence = 0
    }


-- ── Notifications ────────────────────────────────────────────────────────
--
-- Fired when a *background* conversation (not the one you're looking at)
-- reaches a terminal state: response finished, upload done, or error.

type alias ChatNotification =
    { id : Int
    , conversationId : String
    , kind : ChatNotificationKind
    }


type ChatNotificationKind
    = ResponseComplete { preview : String }
    | MediaUploadComplete
    | ChatErrorNotification ChatError


-- ── Chat Page State (top-level) ──────────────────────────────────────────
--
-- This is what sits on the Model. It owns the sidebar, the Dict of
-- per-conversation states, and the notification queue.

type alias ChatPageState =
    { conversations : RemoteData (List Conversation)
    , activeChatId : Maybe String
    , conversationStates : Dict String ChatConversationState
    , notifications : List ChatNotification
    , notificationCounter : Int
    , renamingConversationId : Maybe String
    , renameText : String
    , confirmingDeleteId : Maybe String
    }


emptyChatPageState : ChatPageState
emptyChatPageState =
    { conversations = NotAsked
    , activeChatId = Nothing
    , conversationStates = Dict.empty
    , notifications = []
    , notificationCounter = 0
    , renamingConversationId = Nothing
    , renameText = ""
    , confirmingDeleteId = Nothing
    }


-- ── Chat State Helpers ───────────────────────────────────────────────────


getConversationState : String -> ChatPageState -> ChatConversationState
getConversationState convId page =
    Dict.get convId page.conversationStates
        |> Maybe.withDefault emptyChatConversationState


updateConversationState : String -> (ChatConversationState -> ChatConversationState) -> ChatPageState -> ChatPageState
updateConversationState convId fn page =
    let
        current =
            getConversationState convId page

        updated =
            fn current
    in
    { page | conversationStates = Dict.insert convId updated page.conversationStates }


nextMessageId : ChatConversationState -> ( String, ChatConversationState )
nextMessageId state =
    let
        newCounter = state.messageCounter + 1
    in
    ( "msg-" ++ String.fromInt newCounter
    , { state | messageCounter = newCounter }
    )


nextUploadId : ChatConversationState -> ( String, ChatConversationState )
nextUploadId state =
    let
        newCounter = state.uploadCounter + 1
    in
    ( "upload-" ++ String.fromInt newCounter
    , { state | uploadCounter = newCounter }
    )


enqueueUpload : MediaUploadPayload -> ChatConversationState -> ( String, ChatConversationState )
enqueueUpload media state =
    let
        ( uploadId, state2 ) =
            nextUploadId state

        task =
            { id = uploadId
            , media = media
            , status = Uploading
            }
    in
    ( uploadId, { state2 | uploads = state2.uploads ++ [ task ] } )


updateUpload : String -> UploadStatus -> ChatConversationState -> ChatConversationState
updateUpload uploadId newStatus state =
    let
        updateTask t =
            if t.id == uploadId then
                { t | status = newStatus }
            else
                t
    in
    { state | uploads = List.map updateTask state.uploads }


removeUpload : String -> ChatConversationState -> ChatConversationState
removeUpload uploadId state =
    { state | uploads = List.filter (\t -> t.id /= uploadId) state.uploads }


activeUploads : ChatConversationState -> List UploadTask
activeUploads state =
    List.filter (\t -> t.status == Uploading) state.uploads


failedUploads : ChatConversationState -> List UploadTask
failedUploads state =
    List.filter
        (\t ->
            case t.status of
                UploadFailed _ -> True
                _ -> False
        )
        state.uploads


hasUploadsInProgress : ChatConversationState -> Bool
hasUploadsInProgress state =
    List.any (\t -> t.status == Uploading) state.uploads


addNotification : String -> ChatNotificationKind -> ChatPageState -> ChatPageState
addNotification convId kind page =
    let
        newId = page.notificationCounter + 1

        note =
            { id = newId
            , conversationId = convId
            , kind = kind
            }
    in
    { page
        | notifications = note :: page.notifications
        , notificationCounter = newId
    }


dismissNotification : Int -> ChatPageState -> ChatPageState
dismissNotification noteId page =
    { page | notifications = List.filter (\n -> n.id /= noteId) page.notifications }


dismissNotificationsFor : String -> ChatPageState -> ChatPageState
dismissNotificationsFor convId page =
    { page | notifications = List.filter (\n -> n.conversationId /= convId) page.notifications }


isBackground : String -> ChatPageState -> Bool
isBackground convId page =
    page.activeChatId /= Just convId


-- ── Navigation Guards ────────────────────────────────────────────────────


isRecording : ChatActivity -> Bool
isRecording activity =
    case activity of
        ChatComposing ComposingVoice -> True
        ChatComposing ComposingVideo -> True
        _ -> False


canSwitchConversation : ChatPageState -> Bool
canSwitchConversation page =
    case page.activeChatId of
        Nothing ->
            True

        Just activeId ->
            not (isRecording (getConversationState activeId page).activity)


canNavigateAway : ChatPageState -> Bool
canNavigateAway page =
    canSwitchConversation page


-- ── Activity Queries ─────────────────────────────────────────────────────


hasActiveWork : ChatConversationState -> Bool
hasActiveWork conv =
    hasUploadsInProgress conv
        || (case conv.activity of
                ChatIdle -> False
                ChatComposing _ -> False
                ChatError _ -> False
                ChatObserving -> False
                _ -> True
           )


conversationsWithActiveWork : ChatPageState -> List String
conversationsWithActiveWork page =
    Dict.toList page.conversationStates
        |> List.filterMap
            (\( convId, state ) ->
                if hasActiveWork state then Just convId else Nothing
            )


isStreamingOrWaiting : ChatActivity -> Bool
isStreamingOrWaiting activity =
    case activity of
        ChatStreaming _ -> True
        ChatAwaitingResponse _ -> True
        ChatTranscribing -> True
        _ -> False


canSendMessage : ChatActivity -> Bool
canSendMessage activity =
    case activity of
        ChatComposing ComposingVoice -> False
        ChatComposing ComposingVideo -> False
        ChatObserving -> False
        ChatError _ -> False
        _ -> True


canAttachFile : ChatActivity -> Bool
canAttachFile activity =
    case activity of
        ChatIdle -> True
        ChatComposing ComposingText -> True
        _ -> False


canStartRecording : ChatActivity -> Bool
canStartRecording activity =
    case activity of
        ChatIdle -> True
        ChatComposing ComposingText -> True
        _ -> False


isTelegramConversation : String -> ChatPageState -> Bool
isTelegramConversation convId page =
    case page.conversations of
        Success convs ->
            List.any
                (\c ->
                    c.id == convId && (c.protocol == Just "telegram" || (c.protocol == Nothing && not (isUuidLike c.id)))
                )
                convs

        _ ->
            False


{-| Web conversations have UUID ids, telegram ones have numeric_numeric ids -}
isUuidLike : String -> Bool
isUuidLike s =
    String.length s > 20 && String.contains "-" s


directionFromString : String -> MessageDirection
directionFromString s =
    if s == "outbound" then Outbound else Inbound


directionToString : MessageDirection -> String
directionToString d =
    case d of
        Inbound -> "inbound"
        Outbound -> "outbound"


-- ═══════════════════════════════════════════════════════════════════════════
-- WORK TYPES
-- ═══════════════════════════════════════════════════════════════════════════


type alias WorkProject =
    { id : Int
    , name : String
    , description : String
    , gitRemoteUrl : Maybe String
    , tags : List String
    , isActive : Bool
    , taskCount : Int
    , createdAt : String
    , updatedAt : String
    }


type alias WorkTask =
    { id : Int
    , projectId : Int
    , status : String         -- "todo", "in_progress", "ready_for_review", "under_review", "done", "blocked", "abandoned"
    , priority : String       -- "low", "medium", "high", "critical"
    , sortOrder : Int
    , title : String
    , description : String
    , tags : List String
    , completedAt : Maybe String
    , createdAt : String
    , updatedAt : String
    , blockedBy : List Int
    , blocks : List Int
    }


type alias WorkDocument =
    { id : Int
    , projectId : Int
    , documentType : String   -- "plan", "specification", "notes", "code", "other"
    , title : String
    , content : String
    , version : Int
    , createdAt : String
    , updatedAt : String
    }


type alias WorkComment =
    { id : Int
    , taskId : Maybe Int
    , documentId : Maybe Int
    , parentCommentId : Maybe Int
    , content : String
    , createdAt : String
    , updatedAt : String
    }


type alias ActivityLog =
    { id : Int
    , projectId : Maybe Int
    , taskId : Maybe Int
    , documentId : Maybe Int
    , action : String
    , actor : String
    , details : String
    , createdAt : String
    }


type alias LiveBoard =
    { backlog : List WorkTask
    , selected : List SelectedTask
    , stats : LiveBoardStats
    }


type alias SelectedTask =
    { selection : LiveBoardSelection
    , task : WorkTask
    , comments : List WorkComment
    }


type alias LiveBoardSelection =
    { id : Int
    , taskId : Int
    , sortOrder : Int
    , selectedAt : String
    , startedAt : Maybe String
    , completedAt : Maybe String
    , status : String         -- "queued", "active", "paused", "done", "failed"
    }


type alias LiveBoardStats =
    { totalBacklog : Int
    , totalSelected : Int
    , queued : Int
    , completed : Int
    , failed : Int
    , active : Maybe Int
    , agentLoopState : String -- "idle", "running", "paused"
    }


type alias TaskAnalytics =
    { statusCounts : List StatusCount
    , avgCompletionHours : Maybe Float
    , throughput30d : List DayCount
    }


type alias StatusCount =
    { status : String
    , count : Int
    }


type alias DayCount =
    { date : String
    , count : Int
    }


-- Work form types

type alias ProjectForm =
    { name : String
    , description : String
    , tags : String
    , gitRemoteUrl : String
    }


emptyProjectForm : ProjectForm
emptyProjectForm =
    { name = "", description = "", tags = "", gitRemoteUrl = "" }


type alias TaskForm =
    { title : String
    , description : String
    , priority : String
    , status : String
    }


emptyTaskForm : TaskForm
emptyTaskForm =
    { title = "", description = "", priority = "medium", status = "todo" }


type alias DocumentForm =
    { title : String
    , content : String
    , documentType : String
    }


emptyDocumentForm : DocumentForm
emptyDocumentForm =
    { title = "", content = "", documentType = "notes" }


type alias CommentForm =
    { content : String
    }


emptyCommentForm : CommentForm
emptyCommentForm =
    { content = "" }


-- Inline editing state for task detail

type EditingField
    = NotEditing
    | EditingTitle String
    | EditingDescription String
    | EditingPriority
    | EditingTags String


-- Task view mode for project detail

type TaskViewMode
    = ListView
    | BoardView


-- Tab type for project detail

type ProjectTab
    = TasksTab
    | DocumentsTab
    | ActivityTab


-- Task filter state

type alias TaskFilters =
    { statusFilter : List String
    }


emptyTaskFilters : TaskFilters
emptyTaskFilters =
    { statusFilter = []
    }


-- ═══════════════════════════════════════════════════════════════════════════
-- VOICE TYPES
-- ═══════════════════════════════════════════════════════════════════════════


type VoiceRecordingState
    = VoiceIdle
    | VoiceRecording
    | VoiceTranscribing
    | VoiceFormatting
    | VoiceDone String       -- formatted markdown result
    | VoiceError String


type VoiceMode
    = VoiceTicket
    | VoiceEdit
    | VoiceComment


voiceModeToString : VoiceMode -> String
voiceModeToString mode =
    case mode of
        VoiceTicket -> "ticket"
        VoiceEdit -> "edit"
        VoiceComment -> "comment"


type alias VoiceState =
    { recordingState : VoiceRecordingState
    , mode : VoiceMode
    , transcription : Maybe String
    , existingContent : Maybe String  -- for edit mode
    }


emptyVoiceState : VoiceState
emptyVoiceState =
    { recordingState = VoiceIdle
    , mode = VoiceComment
    , transcription = Nothing
    , existingContent = Nothing
    }


-- ═══════════════════════════════════════════════════════════════════════════
-- WORK HELPERS
-- ═══════════════════════════════════════════════════════════════════════════


taskStatusLabel : String -> String
taskStatusLabel status =
    case status of
        "todo" -> "Todo"
        "in_progress" -> "In Progress"
        "ready_for_review" -> "Ready for Review"
        "under_review" -> "Under Review"
        "done" -> "Done"
        "blocked" -> "Blocked"
        "abandoned" -> "Abandoned"
        _ -> status


taskPriorityLabel : String -> String
taskPriorityLabel priority =
    case priority of
        "low" -> "Low"
        "medium" -> "Medium"
        "high" -> "High"
        "critical" -> "Critical"
        _ -> priority


-- Media URL helper

mediaUrl : String -> String -> String
mediaUrl chatId filename =
    "/api/media/" ++ chatId ++ "/" ++ filename


-- Extract filename from media path like "./data/media/123/file.ogg"

filenameFromPath : String -> Maybe String
filenameFromPath path =
    path
        |> String.split "/"
        |> List.reverse
        |> List.head
