module Api exposing (..)

import Http
import Json.Decode as D
import Json.Encode as E
import Types exposing (..)
import Url.Builder as Url


-- Decoders

promptItemDecoder : D.Decoder PromptItem
promptItemDecoder =
    D.succeed PromptItem
        |> andMap (D.field "id" D.string)
        |> andMap (D.field "source" (D.field "type" D.string))
        |> andMap (D.field "user_id" D.int)
        |> andMap (D.field "prompt" D.string)
        |> andMap (D.maybe (D.field "media_path" D.string))
        |> andMap (D.field "status" D.string)
        |> andMap (D.field "created_at" D.string)
        |> andMap (D.maybe (D.field "started_at" D.string))
        |> andMap (D.maybe (D.field "completed_at" D.string))
        |> andMap (D.maybe (D.field "error" D.string))


feedDecoder : D.Decoder FeedData
feedDecoder =
    D.succeed FeedData
        |> andMap (D.field "pending" (D.list promptItemDecoder))
        |> andMap (D.field "pending_count" D.int)
        |> andMap (D.field "running" (D.nullable promptItemDecoder))
        |> andMap (D.field "recent_completed" (D.list promptItemDecoder))
        |> andMap (D.field "completed_count" D.int)


responseItemDecoder : D.Decoder ResponseItem
responseItemDecoder =
    D.succeed ResponseItem
        |> andMap (D.field "id" D.string)
        |> andMap (D.field "prompt_id" D.string)
        |> andMap (D.field "source" (D.field "type" D.string))
        |> andMap (D.field "user_id" D.int)
        |> andMap (D.field "content" D.string)
        |> andMap (D.field "is_partial" D.bool)
        |> andMap (D.field "is_final" D.bool)
        |> andMap (D.field "sequence" D.int)
        |> andMap (D.field "status" D.string)
        |> andMap (D.field "created_at" D.string)
        |> andMap (D.maybe (D.field "sent_at" D.string))
        |> andMap (D.maybe (D.field "next_attempt_at" D.string))
        |> andMap (D.maybe (D.field "error" D.string))


responsesDecoder : D.Decoder ResponseFeedData
responsesDecoder =
    D.succeed ResponseFeedData
        |> andMap (D.field "pending" (D.list responseItemDecoder))
        |> andMap (D.field "pending_count" D.int)
        |> andMap (D.field "recent_sent" (D.list responseItemDecoder))
        |> andMap (D.field "sent_count" D.int)
        |> andMap (D.field "recent_failed" (D.list responseItemDecoder))
        |> andMap (D.field "failed_count" D.int)


storedMessageDecoder : D.Decoder StoredMessage
storedMessageDecoder =
    D.succeed StoredMessage
        |> andMap (D.field "id" D.string)
        |> andMap (D.field "chat_id" D.string)
        |> andMap (D.maybe (D.field "user_id" D.int))
        |> andMap (D.field "direction" D.string)
        |> andMap (D.field "content" D.string)
        |> andMap (D.maybe (D.field "media_type" D.string))
        |> andMap (D.maybe (D.field "media_path" D.string))
        |> andMap (D.field "timestamp" D.string)


chatSummaryDecoder : D.Decoder ChatSummary
chatSummaryDecoder =
    D.map5 ChatSummary
        (D.field "chat_id" D.string)
        (D.maybe (D.field "topic_id" D.int))
        (D.maybe (D.field "username" D.string))
        (D.maybe (D.field "display_name" D.string))
        (D.field "message_count" D.int)


messagesPageDecoder : D.Decoder MessagesPage
messagesPageDecoder =
    D.map5 MessagesPage
        (D.field "messages" (D.list storedMessageDecoder))
        (D.field "total" D.int)
        (D.field "page" D.int)
        (D.field "page_size" D.int)
        (D.field "total_pages" D.int)


logEntryDecoder : D.Decoder LogEntry
logEntryDecoder =
    D.map4 LogEntry
        (D.field "timestamp" D.string)
        (D.field "level" D.string)
        (D.field "component" D.string)
        (D.field "message" D.string)


logsPageDecoder : D.Decoder LogsPage
logsPageDecoder =
    D.map5 LogsPage
        (D.field "entries" (D.list logEntryDecoder))
        (D.field "total" D.int)
        (D.field "page" D.int)
        (D.field "page_size" D.int)
        (D.field "total_pages" D.int)


andMap : D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
andMap =
    D.map2 (|>)


-- API Calls

getStatus : (Result Http.Error () -> msg) -> Cmd msg
getStatus toMsg =
    Http.get
        { url = "/api/status"
        , expect = Http.expectWhatever toMsg
        }


getFeed : (Result Http.Error FeedData -> msg) -> Cmd msg
getFeed toMsg =
    Http.get
        { url = "/api/feed"
        , expect = Http.expectJson toMsg feedDecoder
        }


getResponses : (Result Http.Error ResponseFeedData -> msg) -> Cmd msg
getResponses toMsg =
    Http.get
        { url = "/api/responses"
        , expect = Http.expectJson toMsg responsesDecoder
        }


getChats : (Result Http.Error (List ChatSummary) -> msg) -> Cmd msg
getChats toMsg =
    Http.get
        { url = "/api/chats"
        , expect = Http.expectJson toMsg (D.field "chats" (D.list chatSummaryDecoder))
        }


getMessages : String -> Int -> Int -> Maybe String -> Maybe String -> (Result Http.Error MessagesPage -> msg) -> Cmd msg
getMessages chatId page pageSize maybeSearch maybeTopicFilter toMsg =
    let
        searchParam =
            case maybeSearch of
                Just search ->
                    if String.isEmpty search then
                        []
                    else
                        [ Url.string "search" search ]
                Nothing ->
                    []

        topicParam =
            case maybeTopicFilter of
                Just tid ->
                    [ Url.string "topic_id" tid ]

                Nothing ->
                    []
    in
    Http.get
        { url = Url.absolute [ "api", "messages", chatId ]
            ([ Url.int "page" page
             , Url.int "page_size" pageSize
             ] ++ searchParam ++ topicParam)
        , expect = Http.expectJson toMsg messagesPageDecoder
        }


getLogs : Int -> Int -> Maybe String -> (Result Http.Error LogsPage -> msg) -> Cmd msg
getLogs page pageSize maybeSearch toMsg =
    let
        searchParam =
            case maybeSearch of
                Just search ->
                    if String.isEmpty search then
                        []
                    else
                        [ Url.string "search" search ]
                Nothing ->
                    []
    in
    Http.get
        { url = Url.absolute [ "api", "logs" ]
            ([ Url.int "page" page
             , Url.int "page_size" pageSize
             ] ++ searchParam)
        , expect = Http.expectJson toMsg logsPageDecoder
        }


-- Semantic Indexer API

taskStatusDecoder : D.Decoder TaskStatus
taskStatusDecoder =
    D.succeed TaskStatus
        |> andMap (D.field "activity" D.string)
        |> andMap (D.maybe (D.field "current_file" D.string))
        |> andMap (D.field "files_indexed" D.int)
        |> andMap (D.field "files_skipped" D.int)
        |> andMap (D.maybe (D.field "files_total" D.int))
        |> andMap (D.field "chunks_processed" D.int)
        |> andMap (D.maybe (D.field "chunks_total" D.int))


semanticStatusDecoder : D.Decoder SemanticStatus
semanticStatusDecoder =
    D.succeed SemanticStatus
        |> andMap (D.field "enabled" D.bool)
        |> andMap (D.field "memory" taskStatusDecoder)
        |> andMap (D.field "conversations" taskStatusDecoder)
        |> andMap (D.field "total_memory_chunks" D.int)
        |> andMap (D.field "total_memory_files" D.int)
        |> andMap (D.field "total_conversation_chunks" D.int)
        |> andMap (D.field "total_conversation_sessions" D.int)
        |> andMap (D.field "total_memory_files_available" D.int)
        |> andMap (D.field "total_conversation_files_available" D.int)
        |> andMap (D.field "memory_files_stale" D.int)
        |> andMap (D.field "conversation_files_stale" D.int)
        |> andMap (D.maybe (D.field "last_conversation_poll_at" D.int))
        |> andMap (D.field "conversation_poll_interval_secs" D.int)


getSemanticStatus : (Result Http.Error SemanticStatus -> msg) -> Cmd msg
getSemanticStatus toMsg =
    Http.get
        { url = "/api/semantic/status"
        , expect = Http.expectJson toMsg semanticStatusDecoder
        }


toggleSemantic : Bool -> (Result Http.Error SemanticStatus -> msg) -> Cmd msg
toggleSemantic enabled toMsg =
    Http.post
        { url = "/api/semantic/toggle"
        , body = Http.jsonBody (E.object [ ( "enabled", E.bool enabled ) ])
        , expect = Http.expectJson toMsg semanticStatusDecoder
        }


triggerSemanticReindex : (Result Http.Error () -> msg) -> Cmd msg
triggerSemanticReindex toMsg =
    Http.post
        { url = "/api/semantic/reindex"
        , body = Http.emptyBody
        , expect = Http.expectWhatever toMsg
        }


-- Tunnel API

tunnelStatusDecoder : D.Decoder TunnelStatus
tunnelStatusDecoder =
    D.succeed TunnelStatus
        |> andMap (D.field "active" D.bool)
        |> andMap (D.maybe (D.field "url" D.string))
        |> andMap (D.maybe (D.field "qr_svg" D.string))


getTunnelStatus : (Result Http.Error TunnelStatus -> msg) -> Cmd msg
getTunnelStatus toMsg =
    Http.get
        { url = "/api/tunnel/status"
        , expect = Http.expectJson toMsg tunnelStatusDecoder
        }


-- Setup API

type alias SetupStatusResponse =
    { dataDir : String
    , hasTelegramToken : Bool
    , hasGeminiKey : Bool
    , hasClaudeCli : Bool
    , claudeCliVersion : Maybe String
    , hasAllowedUsername : Bool
    , hasThreadingEnabled : Bool
    , isComplete : Bool
    , platform : String
    , geminiKeyPreview : Maybe String
    , allowedUsernameValue : Maybe String
    , botName : Maybe String
    }


type alias TelegramSetupResponse =
    { success : Bool
    , botName : Maybe String
    , error : Maybe String
    }


type alias GeminiSetupResponse =
    { success : Bool
    , error : Maybe String
    }


type alias ClaudeInstallResponse =
    { success : Bool
    , version : Maybe String
    , error : Maybe String
    }


setupStatusDecoder : D.Decoder SetupStatusResponse
setupStatusDecoder =
    D.succeed SetupStatusResponse
        |> andMap (D.field "data_dir" D.string)
        |> andMap (D.field "has_telegram_token" D.bool)
        |> andMap (D.field "has_gemini_key" D.bool)
        |> andMap (D.field "has_claude_cli" D.bool)
        |> andMap (D.maybe (D.field "claude_cli_version" D.string))
        |> andMap (D.field "has_allowed_username" D.bool)
        |> andMap (D.field "has_threading_enabled" D.bool)
        |> andMap (D.field "is_complete" D.bool)
        |> andMap (D.field "platform" D.string)
        |> andMap (D.maybe (D.field "gemini_key_preview" D.string))
        |> andMap (D.maybe (D.field "allowed_username_value" D.string))
        |> andMap (D.maybe (D.field "bot_name" D.string))


telegramSetupDecoder : D.Decoder TelegramSetupResponse
telegramSetupDecoder =
    D.map3 TelegramSetupResponse
        (D.field "success" D.bool)
        (D.maybe (D.field "bot_name" D.string))
        (D.maybe (D.field "error" D.string))


geminiSetupDecoder : D.Decoder GeminiSetupResponse
geminiSetupDecoder =
    D.map2 GeminiSetupResponse
        (D.field "success" D.bool)
        (D.maybe (D.field "error" D.string))


type alias ThreadingCheckResponse =
    { success : Bool
    , enabled : Bool
    , error : Maybe String
    }


claudeInstallDecoder : D.Decoder ClaudeInstallResponse
claudeInstallDecoder =
    D.map3 ClaudeInstallResponse
        (D.field "success" D.bool)
        (D.maybe (D.field "version" D.string))
        (D.maybe (D.field "error" D.string))


threadingCheckDecoder : D.Decoder ThreadingCheckResponse
threadingCheckDecoder =
    D.map3 ThreadingCheckResponse
        (D.field "success" D.bool)
        (D.field "enabled" D.bool)
        (D.maybe (D.field "error" D.string))


getSetupStatus : (Result Http.Error SetupStatusResponse -> msg) -> Cmd msg
getSetupStatus toMsg =
    Http.get
        { url = "/api/setup/status"
        , expect = Http.expectJson toMsg setupStatusDecoder
        }


postTelegramToken : String -> (Result Http.Error TelegramSetupResponse -> msg) -> Cmd msg
postTelegramToken token toMsg =
    Http.post
        { url = "/api/setup/telegram"
        , body = Http.jsonBody (E.object [ ( "token", E.string token ) ])
        , expect = Http.expectJson toMsg telegramSetupDecoder
        }


postGeminiKey : String -> (Result Http.Error GeminiSetupResponse -> msg) -> Cmd msg
postGeminiKey key toMsg =
    Http.post
        { url = "/api/setup/gemini"
        , body = Http.jsonBody (E.object [ ( "key", E.string key ) ])
        , expect = Http.expectJson toMsg geminiSetupDecoder
        }


postInstallClaude : (Result Http.Error ClaudeInstallResponse -> msg) -> Cmd msg
postInstallClaude toMsg =
    Http.post
        { url = "/api/setup/install-claude"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg claudeInstallDecoder
        }


-- Claude auth check

type alias ClaudeAuthCheckResponse =
    { installed : Bool
    , version : Maybe String
    , authenticated : Bool
    , authMode : Maybe String
    , accountEmail : Maybe String
    , accountName : Maybe String
    , needsUpdate : Bool
    , latestVersion : Maybe String
    , error : Maybe String
    }


claudeAuthCheckDecoder : D.Decoder ClaudeAuthCheckResponse
claudeAuthCheckDecoder =
    D.succeed ClaudeAuthCheckResponse
        |> andMap (D.field "installed" D.bool)
        |> andMap (D.maybe (D.field "version" D.string))
        |> andMap (D.field "authenticated" D.bool)
        |> andMap (D.maybe (D.field "auth_mode" D.string))
        |> andMap (D.maybe (D.field "account_email" D.string))
        |> andMap (D.maybe (D.field "account_name" D.string))
        |> andMap (D.field "needs_update" D.bool)
        |> andMap (D.maybe (D.field "latest_version" D.string))
        |> andMap (D.maybe (D.field "error" D.string))


getClaudeAuth : (Result Http.Error ClaudeAuthCheckResponse -> msg) -> Cmd msg
getClaudeAuth toMsg =
    Http.get
        { url = "/api/setup/claude-auth"
        , expect = Http.expectJson toMsg claudeAuthCheckDecoder
        }


postUpdateClaude : (Result Http.Error ClaudeInstallResponse -> msg) -> Cmd msg
postUpdateClaude toMsg =
    Http.post
        { url = "/api/setup/update-claude"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg claudeInstallDecoder
        }


-- Claude test

type alias ClaudeTestResponse =
    { success : Bool
    , output : Maybe String
    , error : Maybe String
    }


claudeTestDecoder : D.Decoder ClaudeTestResponse
claudeTestDecoder =
    D.map3 ClaudeTestResponse
        (D.field "success" D.bool)
        (D.maybe (D.field "output" D.string))
        (D.maybe (D.field "error" D.string))


postTestClaude : (Result Http.Error ClaudeTestResponse -> msg) -> Cmd msg
postTestClaude toMsg =
    Http.post
        { url = "/api/setup/test-claude"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg claudeTestDecoder
        }


checkThreading : (Result Http.Error ThreadingCheckResponse -> msg) -> Cmd msg
checkThreading toMsg =
    Http.post
        { url = "/api/setup/check-threading"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg threadingCheckDecoder
        }


-- Settings API

settingsDecoder : D.Decoder Types.Settings
settingsDecoder =
    D.succeed Types.Settings
        |> andMap (D.field "show_tool_messages" D.bool)
        |> andMap (D.field "show_thinking_messages" D.bool)
        |> andMap (D.field "show_tool_results" D.bool)
        |> andMap (D.field "omp_num_threads" D.int)
        |> andMap (D.maybe (D.field "allowed_username" D.string))
        |> andMap (D.field "chat_harness" D.string)
        |> andMap (D.oneOf [ D.field "claude_model" D.string, D.succeed "claude-opus-4-6" ])
        |> andMap (D.oneOf [ D.field "dev_role_prompt" D.string, D.succeed "" ])
        |> andMap (D.oneOf [ D.field "harden_role_prompt" D.string, D.succeed "" ])
        |> andMap (D.oneOf [ D.field "pm_role_prompt" D.string, D.succeed "" ])


-- API Keys

type alias ApiKeysResponse =
    { hasTelegramToken : Bool
    , telegramTokenMasked : Maybe String
    , telegramStatus : Maybe Types.ApiKeyStatus
    , hasGeminiKey : Bool
    , geminiKeyMasked : Maybe String
    , geminiStatus : Maybe Types.ApiKeyStatus
    , claudeCodeStatus : Maybe Types.ClaudeCodeStatus
    , hasUserContacted : Maybe Bool
    }


type alias UpdateApiKeysResponse =
    { success : Bool
    , telegramUpdated : Bool
    , geminiUpdated : Bool
    , telegramError : Maybe String
    , geminiError : Maybe String
    }


apiKeyStatusDecoder : D.Decoder Types.ApiKeyStatus
apiKeyStatusDecoder =
    D.succeed Types.ApiKeyStatus
        |> andMap (D.field "valid" D.bool)
        |> andMap (D.maybe (D.field "error" D.string))
        |> andMap (D.maybe (D.field "info" D.string))


claudeCodeStatusDecoder : D.Decoder Types.ClaudeCodeStatus
claudeCodeStatusDecoder =
    D.succeed Types.ClaudeCodeStatus
        |> andMap (D.field "auth_mode" D.string)
        |> andMap (D.maybe (D.field "account_email" D.string))
        |> andMap (D.maybe (D.field "account_name" D.string))
        |> andMap (D.maybe (D.field "organization" D.string))


apiKeysDecoder : D.Decoder ApiKeysResponse
apiKeysDecoder =
    D.succeed ApiKeysResponse
        |> andMap (D.field "has_telegram_token" D.bool)
        |> andMap (D.maybe (D.field "telegram_token_masked" D.string))
        |> andMap (D.maybe (D.field "telegram_status" apiKeyStatusDecoder))
        |> andMap (D.field "has_gemini_key" D.bool)
        |> andMap (D.maybe (D.field "gemini_key_masked" D.string))
        |> andMap (D.maybe (D.field "gemini_status" apiKeyStatusDecoder))
        |> andMap (D.maybe (D.field "claude_code_status" claudeCodeStatusDecoder))
        |> andMap (D.maybe (D.field "has_user_contacted" D.bool))


updateApiKeysDecoder : D.Decoder UpdateApiKeysResponse
updateApiKeysDecoder =
    D.succeed UpdateApiKeysResponse
        |> andMap (D.field "success" D.bool)
        |> andMap (D.field "telegram_updated" D.bool)
        |> andMap (D.field "gemini_updated" D.bool)
        |> andMap (D.maybe (D.field "telegram_error" D.string))
        |> andMap (D.maybe (D.field "gemini_error" D.string))


getApiKeys : (Result Http.Error ApiKeysResponse -> msg) -> Cmd msg
getApiKeys toMsg =
    Http.get
        { url = "/api/setup/api-keys"
        , expect = Http.expectJson toMsg apiKeysDecoder
        }


putApiKeys : Maybe String -> Maybe String -> (Result Http.Error UpdateApiKeysResponse -> msg) -> Cmd msg
putApiKeys telegramToken geminiKey toMsg =
    Http.request
        { method = "PUT"
        , headers = []
        , url = "/api/setup/api-keys"
        , body = Http.jsonBody
            (E.object
                (List.filterMap identity
                    [ telegramToken |> Maybe.map (\t -> ( "telegram_token", E.string t ))
                    , geminiKey |> Maybe.map (\k -> ( "gemini_key", E.string k ))
                    ]
                )
            )
        , expect = Http.expectJson toMsg updateApiKeysDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


getSettings : (Result Http.Error Types.Settings -> msg) -> Cmd msg
getSettings toMsg =
    Http.get
        { url = "/api/settings"
        , expect = Http.expectJson toMsg settingsDecoder
        }


putSettings : Types.Settings -> (Result Http.Error Types.Settings -> msg) -> Cmd msg
putSettings settings toMsg =
    Http.request
        { method = "PUT"
        , headers = []
        , url = "/api/settings"
        , body = Http.jsonBody
            (E.object
                ([ ( "show_tool_messages", E.bool settings.showToolMessages )
                 , ( "show_thinking_messages", E.bool settings.showThinkingMessages )
                 , ( "show_tool_results", E.bool settings.showToolResults )
                 , ( "omp_num_threads", E.int settings.ompNumThreads )
                 , ( "chat_harness", E.string settings.chatHarness )
                 , ( "claude_model", E.string settings.claudeModel )
                 , ( "dev_role_prompt", E.string settings.devRolePrompt )
                 , ( "harden_role_prompt", E.string settings.hardenRolePrompt )
                 , ( "pm_role_prompt", E.string settings.pmRolePrompt )
                 ]
                    ++ (case settings.allowedUsername of
                            Just name ->
                                [ ( "allowed_username", E.string name ) ]

                            Nothing ->
                                [ ( "allowed_username", E.null ) ]
                       )
                )
            )
        , expect = Http.expectJson toMsg settingsDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


-- Cron Jobs API

cronJobDecoder : D.Decoder Types.CronJob
cronJobDecoder =
    D.succeed Types.CronJob
        |> andMap (D.field "id" D.string)
        |> andMap (D.maybe (D.field "name" D.string))
        |> andMap (D.field "schedule" D.string)
        |> andMap (D.field "status" D.string)
        |> andMap (D.field "deferrable" D.bool)
        |> andMap (D.maybe (D.field "next_run" D.string))
        |> andMap (D.maybe (D.field "last_run" D.string))
        |> andMap (D.field "created_at" D.string)


cronStatusDecoder : D.Decoder Types.CronStatus
cronStatusDecoder =
    D.map3 Types.CronStatus
        (D.field "active_jobs" D.int)
        (D.field "paused_jobs" D.int)
        (D.field "waiting_executions" D.int)


getCronJobs : (Result Http.Error (List Types.CronJob) -> msg) -> Cmd msg
getCronJobs toMsg =
    Http.get
        { url = "/api/cron/jobs"
        , expect = Http.expectJson toMsg (D.field "jobs" (D.list cronJobDecoder))
        }


getCronStatus : (Result Http.Error Types.CronStatus -> msg) -> Cmd msg
getCronStatus toMsg =
    Http.get
        { url = "/api/cron/status"
        , expect = Http.expectJson toMsg cronStatusDecoder
        }


pauseCronJob : String -> (Result Http.Error Types.CronJob -> msg) -> Cmd msg
pauseCronJob jobId toMsg =
    Http.post
        { url = "/api/cron/jobs/" ++ jobId ++ "/pause"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg cronJobDecoder
        }


resumeCronJob : String -> (Result Http.Error Types.CronJob -> msg) -> Cmd msg
resumeCronJob jobId toMsg =
    Http.post
        { url = "/api/cron/jobs/" ++ jobId ++ "/resume"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg cronJobDecoder
        }


cancelCronJob : String -> (Result Http.Error Types.CronJob -> msg) -> Cmd msg
cancelCronJob jobId toMsg =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = "/api/cron/jobs/" ++ jobId
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg cronJobDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


-- ═══════════════════════════════════════════════════════════════════════════
-- CHAT API
-- ═══════════════════════════════════════════════════════════════════════════


conversationDecoder : D.Decoder Types.Conversation
conversationDecoder =
    D.succeed Types.Conversation
        |> andMap (D.field "id" D.string)
        |> andMap (D.field "name" D.string)
        |> andMap (D.maybe (D.field "custom_name" D.string))
        |> andMap (D.maybe (D.field "auto_name" D.string))
        |> andMap (D.maybe (D.field "display_name" D.string))
        |> andMap (D.maybe (D.field "protocol" D.string))
        |> andMap (D.maybe (D.field "last_message_preview" D.string))
        |> andMap (D.field "updated_at" D.string)


chatMessageDecoder : D.Decoder Types.ChatMessage
chatMessageDecoder =
    D.map5
        (\id dir content ts attachments ->
            { id = id
            , direction = Types.directionFromString dir
            , content = content
            , timestamp = ts
            , attachments = attachments
            }
        )
        (D.field "id" D.string)
        (D.field "direction" D.string)
        (D.field "content" D.string)
        (D.field "timestamp" D.string)
        (D.map2 Types.mediaAttachmentFromServerFields
            (D.maybe (D.field "media_type" D.string))
            (D.maybe (D.field "media_path" D.string))
        )


getConversations : (Result Http.Error (List Types.Conversation) -> msg) -> Cmd msg
getConversations toMsg =
    Http.get
        { url = "/api/chat/conversations"
        , expect = Http.expectJson toMsg (D.field "conversations" (D.list conversationDecoder))
        }


getChatMessages : String -> (Result Http.Error (List Types.ChatMessage) -> msg) -> Cmd msg
getChatMessages conversationId toMsg =
    Http.get
        { url = "/api/chat/messages/" ++ conversationId
        , expect = Http.expectJson toMsg (D.field "messages" (D.list chatMessageDecoder))
        }


type alias CreateConversationResponse =
    { conversationId : String
    }


createConversation : (Result Http.Error CreateConversationResponse -> msg) -> Cmd msg
createConversation toMsg =
    Http.post
        { url = "/api/chat/conversations"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg
            (D.map CreateConversationResponse
                (D.field "conversation_id" D.string)
            )
        }


type alias SendMessageResponse =
    { messageId : String
    , status : String
    }


sendChatMessage : String -> String -> (Result Http.Error SendMessageResponse -> msg) -> Cmd msg
sendChatMessage conversationId content toMsg =
    Http.post
        { url = "/api/chat/send"
        , body = Http.jsonBody
            (E.object
                [ ( "conversation_id", E.string conversationId )
                , ( "content", E.string content )
                ]
            )
        , expect = Http.expectJson toMsg
            (D.map2 SendMessageResponse
                (D.field "message_id" D.string)
                (D.field "status" D.string)
            )
        }


renameConversation : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
renameConversation conversationId name toMsg =
    Http.request
        { method = "PUT"
        , headers = []
        , url = "/api/chat/conversations/" ++ conversationId ++ "/name"
        , body = Http.jsonBody (E.object [ ( "name", E.string name ) ])
        , expect = Http.expectWhatever toMsg
        , timeout = Nothing
        , tracker = Nothing
        }


deleteConversation : String -> (Result Http.Error () -> msg) -> Cmd msg
deleteConversation conversationId toMsg =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = "/api/chat/conversations/" ++ conversationId
        , body = Http.emptyBody
        , expect = Http.expectWhatever toMsg
        , timeout = Nothing
        , tracker = Nothing
        }


type alias UploadMediaResponse =
    { success : Bool
    , messageId : Maybe String
    , transcription : Maybe String
    , mediaType : Maybe String
    , mediaPath : Maybe String
    , error : Maybe String
    }


uploadMediaResponseDecoder : D.Decoder UploadMediaResponse
uploadMediaResponseDecoder =
    D.succeed UploadMediaResponse
        |> andMap (D.field "success" D.bool)
        |> andMap (D.maybe (D.field "message_id" D.string))
        |> andMap (D.maybe (D.field "transcription" D.string))
        |> andMap (D.maybe (D.field "media_type" D.string))
        |> andMap (D.maybe (D.field "media_path" D.string))
        |> andMap (D.maybe (D.field "error" D.string))


uploadChatMedia : String -> String -> String -> String -> String -> (String -> Result Http.Error UploadMediaResponse -> msg) -> Cmd msg
uploadChatMedia conversationId uploadId base64Data filename mimeType toMsg =
    Http.post
        { url = "/api/chat/upload"
        , body = Http.jsonBody
            (E.object
                [ ( "conversation_id", E.string conversationId )
                , ( "data", E.string base64Data )
                , ( "filename", E.string filename )
                , ( "mime_type", E.string mimeType )
                ]
            )
        , expect = Http.expectJson (toMsg uploadId) uploadMediaResponseDecoder
        }


-- ═══════════════════════════════════════════════════════════════════════════
-- WORK API
-- ═══════════════════════════════════════════════════════════════════════════


-- Decoders

workProjectDecoder : D.Decoder Types.WorkProject
workProjectDecoder =
    D.succeed Types.WorkProject
        |> andMap (D.field "id" D.int)
        |> andMap (D.field "name" D.string)
        |> andMap (D.field "description" D.string)
        |> andMap (D.maybe (D.field "git_remote_url" D.string))
        |> andMap (D.field "tags" (D.list D.string))
        |> andMap (D.field "is_active" D.bool)
        |> andMap (D.oneOf [ D.field "task_count" D.int, D.succeed 0 ])
        |> andMap (D.field "created_at" D.string)
        |> andMap (D.field "updated_at" D.string)


workTaskDecoder : D.Decoder Types.WorkTask
workTaskDecoder =
    D.succeed Types.WorkTask
        |> andMap (D.field "id" D.int)
        |> andMap (D.field "project_id" D.int)
        |> andMap (D.field "status" D.string)
        |> andMap (D.field "priority" D.string)
        |> andMap (D.field "sort_order" D.int)
        |> andMap (D.field "title" D.string)
        |> andMap (D.field "description" D.string)
        |> andMap (D.field "tags" (D.list D.string))
        |> andMap (D.maybe (D.field "completed_at" D.string))
        |> andMap (D.field "created_at" D.string)
        |> andMap (D.field "updated_at" D.string)
        |> andMap (D.oneOf [ D.field "blocked_by" (D.list D.int), D.succeed [] ])
        |> andMap (D.oneOf [ D.field "blocks" (D.list D.int), D.succeed [] ])


workDocumentDecoder : D.Decoder Types.WorkDocument
workDocumentDecoder =
    D.succeed Types.WorkDocument
        |> andMap (D.field "id" D.int)
        |> andMap (D.field "project_id" D.int)
        |> andMap (D.field "document_type" D.string)
        |> andMap (D.field "title" D.string)
        |> andMap (D.field "content" D.string)
        |> andMap (D.field "version" D.int)
        |> andMap (D.field "created_at" D.string)
        |> andMap (D.field "updated_at" D.string)


workCommentDecoder : D.Decoder Types.WorkComment
workCommentDecoder =
    D.succeed Types.WorkComment
        |> andMap (D.field "id" D.int)
        |> andMap (D.maybe (D.field "task_id" D.int))
        |> andMap (D.maybe (D.field "document_id" D.int))
        |> andMap (D.maybe (D.field "parent_comment_id" D.int))
        |> andMap (D.field "content" D.string)
        |> andMap (D.field "created_at" D.string)
        |> andMap (D.field "updated_at" D.string)


activityLogDecoder : D.Decoder Types.ActivityLog
activityLogDecoder =
    D.succeed Types.ActivityLog
        |> andMap (D.field "id" D.int)
        |> andMap (D.maybe (D.field "project_id" D.int))
        |> andMap (D.maybe (D.field "task_id" D.int))
        |> andMap (D.maybe (D.field "document_id" D.int))
        |> andMap (D.field "action" D.string)
        |> andMap (D.field "actor" D.string)
        |> andMap (D.field "details" D.string)
        |> andMap (D.field "created_at" D.string)


liveBoardDecoder : D.Decoder Types.LiveBoard
liveBoardDecoder =
    D.map3 Types.LiveBoard
        (D.field "backlog" (D.list workTaskDecoder))
        (D.field "selected" (D.list selectedTaskDecoder))
        (D.field "stats" liveBoardStatsDecoder)


selectedTaskDecoder : D.Decoder Types.SelectedTask
selectedTaskDecoder =
    D.map3 Types.SelectedTask
        (D.field "selection" liveBoardSelectionDecoder)
        (D.field "task" workTaskDecoder)
        (D.field "comments" (D.list workCommentDecoder))


liveBoardSelectionDecoder : D.Decoder Types.LiveBoardSelection
liveBoardSelectionDecoder =
    D.succeed Types.LiveBoardSelection
        |> andMap (D.field "id" D.int)
        |> andMap (D.field "task_id" D.int)
        |> andMap (D.field "sort_order" D.int)
        |> andMap (D.field "selected_at" D.string)
        |> andMap (D.maybe (D.field "started_at" D.string))
        |> andMap (D.maybe (D.field "completed_at" D.string))
        |> andMap (D.field "status" D.string)


liveBoardStatsDecoder : D.Decoder Types.LiveBoardStats
liveBoardStatsDecoder =
    D.succeed Types.LiveBoardStats
        |> andMap (D.field "total_backlog" D.int)
        |> andMap (D.field "total_selected" D.int)
        |> andMap (D.field "queued" D.int)
        |> andMap (D.field "completed" D.int)
        |> andMap (D.field "failed" D.int)
        |> andMap (D.maybe (D.field "active" D.int))
        |> andMap (D.field "agent_loop_state" D.string)


taskAnalyticsDecoder : D.Decoder Types.TaskAnalytics
taskAnalyticsDecoder =
    D.map3 Types.TaskAnalytics
        (D.field "status_counts" (D.list statusCountDecoder))
        (D.maybe (D.field "avg_completion_hours" D.float))
        (D.field "throughput_30d" (D.list dayCountDecoder))


statusCountDecoder : D.Decoder Types.StatusCount
statusCountDecoder =
    D.map2 Types.StatusCount
        (D.field "status" D.string)
        (D.field "count" D.int)


dayCountDecoder : D.Decoder Types.DayCount
dayCountDecoder =
    D.map2 Types.DayCount
        (D.field "date" D.string)
        (D.field "count" D.int)


-- API calls — all POST with JSON bodies to /api/work/...

workPost : String -> E.Value -> D.Decoder a -> (Result Http.Error a -> msg) -> Cmd msg
workPost path body decoder toMsg =
    Http.post
        { url = "/api/work" ++ path
        , body = Http.jsonBody body
        , expect = Http.expectJson toMsg decoder
        }


workPostNoBody : String -> D.Decoder a -> (Result Http.Error a -> msg) -> Cmd msg
workPostNoBody path decoder toMsg =
    Http.post
        { url = "/api/work" ++ path
        , body = Http.jsonBody (E.object [])
        , expect = Http.expectJson toMsg decoder
        }


-- Projects

listProjects : (Result Http.Error (List Types.WorkProject) -> msg) -> Cmd msg
listProjects toMsg =
    workPost "/projects/list"
        (E.object [ ( "active_only", E.bool True ), ( "limit", E.int 100 ) ])
        (D.field "data" (D.list workProjectDecoder))
        toMsg


getProject : Int -> (Result Http.Error Types.WorkProject -> msg) -> Cmd msg
getProject id toMsg =
    workPost "/projects/get"
        (E.object [ ( "project_id", E.int id ) ])
        (D.field "data" workProjectDecoder)
        toMsg


createProject : String -> String -> List String -> Maybe String -> (Result Http.Error Types.WorkProject -> msg) -> Cmd msg
createProject name description tags gitRemoteUrl toMsg =
    let
        fields =
            List.filterMap identity
                [ Just ( "name", E.string name )
                , Just ( "description", E.string description )
                , Just ( "tags", E.list E.string tags )
                , gitRemoteUrl |> Maybe.map (\url -> ( "git_remote_url", E.string url ))
                ]
    in
    workPost "/projects/create"
        (E.object fields)
        (D.field "data" workProjectDecoder)
        toMsg


-- Tasks

listTasks : Maybe Int -> Maybe (List String) -> (Result Http.Error (List Types.WorkTask) -> msg) -> Cmd msg
listTasks projectId statusFilter toMsg =
    let
        fields =
            List.filterMap identity
                [ projectId |> Maybe.map (\pid -> ( "project_id", E.int pid ))
                , statusFilter |> Maybe.map (\sf -> ( "status", E.list E.string sf ))
                , Just ( "limit", E.int 200 )
                ]
    in
    workPost "/tasks/list"
        (E.object fields)
        (D.field "data" (D.list workTaskDecoder))
        toMsg


getTask : Int -> (Result Http.Error Types.WorkTask -> msg) -> Cmd msg
getTask id toMsg =
    workPost "/tasks/get"
        (E.object [ ( "task_id", E.int id ) ])
        (D.field "data" workTaskDecoder)
        toMsg


createTask : Int -> String -> String -> String -> List String -> (Result Http.Error Types.WorkTask -> msg) -> Cmd msg
createTask projectId title description priority tags toMsg =
    workPost "/tasks/create"
        (E.object
            [ ( "project_id", E.int projectId )
            , ( "title", E.string title )
            , ( "description", E.string description )
            , ( "priority", E.string priority )
            , ( "tags", E.list E.string tags )
            ]
        )
        (D.field "data" workTaskDecoder)
        toMsg


updateTask : Int -> { title : Maybe String, description : Maybe String, status : Maybe String, priority : Maybe String, tags : Maybe (List String), comment : Maybe String } -> (Result Http.Error Types.WorkTask -> msg) -> Cmd msg
updateTask taskId fields toMsg =
    let
        optionalFields =
            List.filterMap identity
                [ fields.title |> Maybe.map (\v -> ( "title", E.string v ))
                , fields.description |> Maybe.map (\v -> ( "description", E.string v ))
                , fields.status |> Maybe.map (\v -> ( "status", E.string v ))
                , fields.priority |> Maybe.map (\v -> ( "priority", E.string v ))
                , fields.tags |> Maybe.map (\v -> ( "tags", E.list E.string v ))
                , fields.comment |> Maybe.map (\v -> ( "comment", E.string v ))
                ]
    in
    workPost "/tasks/update"
        (E.object (( "task_id", E.int taskId ) :: optionalFields))
        (D.field "data" workTaskDecoder)
        toMsg


getTaskAnalytics : Maybe Int -> (Result Http.Error Types.TaskAnalytics -> msg) -> Cmd msg
getTaskAnalytics projectId toMsg =
    let
        fields =
            case projectId of
                Just pid -> [ ( "project_id", E.int pid ) ]
                Nothing -> []
    in
    workPost "/tasks/analytics"
        (E.object fields)
        (D.field "data" taskAnalyticsDecoder)
        toMsg


-- Documents

searchDocuments : String -> Maybe Int -> (Result Http.Error (List Types.WorkDocument) -> msg) -> Cmd msg
searchDocuments query projectId toMsg =
    let
        fields =
            [ ( "query", E.string query ), ( "limit", E.int 50 ) ]
                ++ (case projectId of
                        Just pid -> [ ( "project_id", E.int pid ) ]
                        Nothing -> []
                   )
    in
    workPost "/documents/search"
        (E.object fields)
        (D.field "data" (D.list workDocumentDecoder))
        toMsg


getDocument : Int -> (Result Http.Error Types.WorkDocument -> msg) -> Cmd msg
getDocument id toMsg =
    workPost "/documents/get"
        (E.object [ ( "document_id", E.int id ) ])
        (D.field "data" workDocumentDecoder)
        toMsg


createDocument : Int -> String -> String -> String -> (Result Http.Error Types.WorkDocument -> msg) -> Cmd msg
createDocument projectId title content docType toMsg =
    workPost "/documents/create"
        (E.object
            [ ( "project_id", E.int projectId )
            , ( "title", E.string title )
            , ( "content", E.string content )
            , ( "type", E.string docType )
            ]
        )
        (D.field "data" workDocumentDecoder)
        toMsg


-- Comments

listComments : Int -> (Result Http.Error (List Types.WorkComment) -> msg) -> Cmd msg
listComments taskId toMsg =
    workPost "/comments/list"
        (E.object [ ( "task_id", E.int taskId ), ( "limit", E.int 100 ) ])
        (D.field "data" (D.list workCommentDecoder))
        toMsg


listCommentsForDocument : Int -> (Result Http.Error (List Types.WorkComment) -> msg) -> Cmd msg
listCommentsForDocument documentId toMsg =
    workPost "/comments/list"
        (E.object [ ( "document_id", E.int documentId ), ( "limit", E.int 100 ) ])
        (D.field "data" (D.list workCommentDecoder))
        toMsg


upsertComment : { commentId : Maybe Int, taskId : Maybe Int, documentId : Maybe Int, content : String, parentCommentId : Maybe Int } -> (Result Http.Error Types.WorkComment -> msg) -> Cmd msg
upsertComment fields toMsg =
    let
        optionalFields =
            List.filterMap identity
                [ fields.commentId |> Maybe.map (\v -> ( "comment_id", E.int v ))
                , fields.taskId |> Maybe.map (\v -> ( "task_id", E.int v ))
                , fields.documentId |> Maybe.map (\v -> ( "document_id", E.int v ))
                , fields.parentCommentId |> Maybe.map (\v -> ( "parent_comment_id", E.int v ))
                ]
    in
    workPost "/comments/upsert"
        (E.object (( "content", E.string fields.content ) :: optionalFields))
        (D.field "data" workCommentDecoder)
        toMsg


-- Activity

getRecentActivity : Int -> (Result Http.Error (List Types.ActivityLog) -> msg) -> Cmd msg
getRecentActivity limit toMsg =
    workPost "/activity/recent"
        (E.object [ ( "limit", E.int limit ) ])
        (D.field "data" (D.list activityLogDecoder))
        toMsg


-- Live Board

getLiveBoard : (Result Http.Error Types.LiveBoard -> msg) -> Cmd msg
getLiveBoard toMsg =
    workPost "/live-board/get"
        (E.object [ ( "backlog_limit", E.int 50 ) ])
        (D.field "data" liveBoardDecoder)
        toMsg


selectTasks : List Int -> (Result Http.Error (List Types.LiveBoardSelection) -> msg) -> Cmd msg
selectTasks taskIds toMsg =
    workPost "/live-board/select"
        (E.object [ ( "task_ids", E.list E.int taskIds ) ])
        (D.field "data" (D.list liveBoardSelectionDecoder))
        toMsg


deselectTask : Int -> (Result Http.Error () -> msg) -> Cmd msg
deselectTask taskId toMsg =
    Http.post
        { url = "/api/work/live-board/deselect"
        , body = Http.jsonBody (E.object [ ( "task_id", E.int taskId ) ])
        , expect = Http.expectWhatever toMsg
        }


clearCompleted : (Result Http.Error Int -> msg) -> Cmd msg
clearCompleted toMsg =
    workPostNoBody "/live-board/clear-completed"
        (D.at [ "data", "cleared" ] D.int)
        toMsg


moveSelection : Int -> String -> (Result Http.Error () -> msg) -> Cmd msg
moveSelection taskId position toMsg =
    Http.post
        { url = "/api/work/live-board/move"
        , body = Http.jsonBody
            (E.object
                [ ( "task_id", E.int taskId )
                , ( "position", E.string position )
                ]
            )
        , expect = Http.expectWhatever toMsg
        }


takeNextTask : Int -> Bool -> (Result Http.Error () -> msg) -> Cmd msg
takeNextTask projectId force toMsg =
    Http.post
        { url = "/api/work/tasks/take-next"
        , body = Http.jsonBody
            (E.object
                [ ( "project_id", E.int projectId )
                , ( "force", E.bool force )
                ]
            )
        , expect = Http.expectWhatever toMsg
        }


takeNextReviewTask : Int -> Bool -> (Result Http.Error () -> msg) -> Cmd msg
takeNextReviewTask projectId force toMsg =
    Http.post
        { url = "/api/work/tasks/take-next-review"
        , body = Http.jsonBody
            (E.object
                [ ( "project_id", E.int projectId )
                , ( "force", E.bool force )
                ]
            )
        , expect = Http.expectWhatever toMsg
        }


moveTaskToTopOrBottom : Int -> String -> (Result Http.Error Types.WorkTask -> msg) -> Cmd msg
moveTaskToTopOrBottom taskId position toMsg =
    workPost "/tasks/move"
        (E.object
            [ ( "task_id", E.int taskId )
            , ( "position", E.string position )
            ]
        )
        (D.field "data" workTaskDecoder)
        toMsg


rejectReview : Int -> String -> (Result Http.Error Types.WorkTask -> msg) -> Cmd msg
rejectReview taskId reviewerComment toMsg =
    workPost "/tasks/reject-review"
        (E.object
            [ ( "task_id", E.int taskId )
            , ( "reviewer_comment", E.string reviewerComment )
            ]
        )
        (D.field "data" workTaskDecoder)
        toMsg


startAgentLoop : (Result Http.Error String -> msg) -> Cmd msg
startAgentLoop toMsg =
    workPostNoBody "/live-board/agent/start"
        (D.field "message" D.string)
        toMsg


stopAgentLoop : (Result Http.Error String -> msg) -> Cmd msg
stopAgentLoop toMsg =
    workPostNoBody "/live-board/agent/stop"
        (D.field "message" D.string)
        toMsg


ensureAgentLoop : (Result Http.Error String -> msg) -> Cmd msg
ensureAgentLoop toMsg =
    workPost "/live-board/agent/ensure"
        (E.object [ ( "auto_select_from_todo", E.bool True ) ])
        (D.field "message" D.string)
        toMsg


-- ═══════════════════════════════════════════════════════════════════════════
-- VOICE API
-- ═══════════════════════════════════════════════════════════════════════════


type alias TranscribeResult =
    { success : Bool
    , transcription : Maybe String
    , error : Maybe String
    }


type alias FormatResult =
    { success : Bool
    , formatted : Maybe String
    , title : Maybe String
    , error : Maybe String
    }


transcribeResultDecoder : D.Decoder TranscribeResult
transcribeResultDecoder =
    D.map3 TranscribeResult
        (D.field "success" D.bool)
        (D.maybe (D.field "transcription" D.string))
        (D.maybe (D.field "error" D.string))


formatResultDecoder : D.Decoder FormatResult
formatResultDecoder =
    D.succeed FormatResult
        |> andMap (D.field "success" D.bool)
        |> andMap (D.maybe (D.field "formatted" D.string))
        |> andMap (D.maybe (D.field "title" D.string))
        |> andMap (D.maybe (D.field "error" D.string))


transcribeAudio : String -> String -> (Result Http.Error TranscribeResult -> msg) -> Cmd msg
transcribeAudio audioData mimeType toMsg =
    Http.post
        { url = "/api/voice/transcribe"
        , body = Http.jsonBody
            (E.object
                [ ( "audio_data", E.string audioData )
                , ( "mime_type", E.string mimeType )
                ]
            )
        , expect = Http.expectJson toMsg transcribeResultDecoder
        }


formatTranscription : String -> String -> Maybe String -> (Result Http.Error FormatResult -> msg) -> Cmd msg
formatTranscription transcription mode existingContent toMsg =
    let
        baseFields =
            [ ( "transcription", E.string transcription )
            , ( "mode", E.string mode )
            ]

        fields =
            case existingContent of
                Just content ->
                    baseFields ++ [ ( "existing_content", E.string content ) ]

                Nothing ->
                    baseFields
    in
    Http.post
        { url = "/api/voice/format"
        , body = Http.jsonBody (E.object fields)
        , expect = Http.expectJson toMsg formatResultDecoder
        }
