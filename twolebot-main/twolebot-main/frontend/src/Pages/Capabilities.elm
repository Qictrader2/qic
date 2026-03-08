module Pages.Capabilities exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import UI


view : Html msg
view =
    div []
        [ UI.pageHeader "System Capabilities" []
        , UI.col "2.5rem"
            [ viewHeroSection
            , viewTelegramSection
            , viewDashboardSection
            , viewSemanticSection
            , viewCronSection
            , viewMcpSection
            , viewConfigSection
            , viewStorageSection
            , viewApiSection
            ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- HERO SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewHeroSection : Html msg
viewHeroSection =
    div
        [ style "background" ("linear-gradient(135deg, " ++ UI.colors.bgTertiary ++ " 0%, " ++ UI.colors.bgSecondary ++ " 100%)")
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "6px"
        , style "padding" "2.5rem"
        , style "position" "relative"
        , style "overflow" "hidden"
        ]
        [ -- Decorative grid overlay
          div
            [ style "position" "absolute"
            , style "top" "0"
            , style "left" "0"
            , style "right" "0"
            , style "bottom" "0"
            , style "background-image" ("repeating-linear-gradient(0deg, transparent, transparent 40px, " ++ UI.colors.gridLine ++ " 40px, " ++ UI.colors.gridLine ++ " 41px), repeating-linear-gradient(90deg, transparent, transparent 40px, " ++ UI.colors.gridLine ++ " 40px, " ++ UI.colors.gridLine ++ " 41px)")
            , style "opacity" "0.5"
            , style "pointer-events" "none"
            ]
            []
        , -- Accent glow
          div
            [ style "position" "absolute"
            , style "top" "-50px"
            , style "right" "-50px"
            , style "width" "200px"
            , style "height" "200px"
            , style "background" ("radial-gradient(circle, " ++ UI.colors.accentGlow ++ " 0%, transparent 70%)")
            , style "pointer-events" "none"
            ]
            []
        , div [ style "position" "relative" ]
            [ div
                [ style "display" "flex"
                , style "align-items" "center"
                , style "gap" "1rem"
                , style "margin-bottom" "1.5rem"
                ]
                [ div
                    [ style "width" "48px"
                    , style "height" "48px"
                    , style "background" ("linear-gradient(135deg, " ++ UI.colors.accent ++ " 0%, #00a884 100%)")
                    , style "clip-path" "polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)"
                    ]
                    []
                , h2
                    [ style "font-family" UI.fontDisplay
                    , style "font-size" "2rem"
                    , style "font-weight" "700"
                    , style "letter-spacing" "0.05em"
                    , style "color" UI.colors.textPrimary
                    , style "margin" "0"
                    ]
                    [ text "TWOLEBOT" ]
                ]
            , p
                [ style "font-size" "1.125rem"
                , style "color" UI.colors.textSecondary
                , style "margin" "0 0 1.5rem 0"
                , style "max-width" "600px"
                , style "line-height" "1.7"
                ]
                [ text "A self-hosted Telegram bot powered by Claude AI. Send messages, voice notes, or images and get intelligent responses. Schedule automated tasks, maintain persistent memory, and search conversation history." ]
            , div
                [ style "display" "flex"
                , style "gap" "1rem"
                , style "flex-wrap" "wrap"
                ]
                [ featurePill "Telegram Bot"
                , featurePill "Claude AI"
                , featurePill "Voice Transcription"
                , featurePill "Cron Scheduling"
                , featurePill "Semantic Search"
                , featurePill "MCP Server"
                ]
            ]
        ]


featurePill : String -> Html msg
featurePill label =
    span
        [ style "display" "inline-flex"
        , style "align-items" "center"
        , style "gap" "0.5rem"
        , style "padding" "0.5rem 1rem"
        , style "background-color" UI.colors.accentDim
        , style "border" ("1px solid " ++ UI.colors.accent)
        , style "border-radius" "2px"
        , style "font-family" UI.fontMono
        , style "font-size" "0.75rem"
        , style "font-weight" "600"
        , style "color" UI.colors.accent
        , style "letter-spacing" "0.05em"
        , style "text-transform" "uppercase"
        ]
        [ div
            [ style "width" "6px"
            , style "height" "6px"
            , style "background-color" UI.colors.accent
            , style "border-radius" "1px"
            , style "box-shadow" ("0 0 6px " ++ UI.colors.accent)
            ]
            []
        , text label
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- TELEGRAM SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewTelegramSection : Html msg
viewTelegramSection =
    sectionCard "Telegram Bot Features" "telegram"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "Interact with Claude through Telegram. Send text, voice notes, or images for intelligent responses." ]
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(280px, 1fr))"
            , style "gap" "1rem"
            ]
            [ featureItem "Text Messages" "Send any text message and receive Claude's response. Full conversation context is maintained." UI.colors.accent
            , featureItem "Voice Notes" "Send voice messages - automatically transcribed using Google Gemini and processed by Claude." UI.colors.success
            , featureItem "Images" "Send photos for analysis. Gemini extracts visual content which Claude can discuss." UI.colors.warning
            , featureItem "/clear Command" "Reset conversation context to start fresh. Useful for changing topics." UI.colors.error
            ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- DASHBOARD SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewDashboardSection : Html msg
viewDashboardSection =
    sectionCard "Web Dashboard" "dashboard"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "Monitor and manage your bot through the web interface at "
            , codeInline "http://localhost:8080"
            , text "."
            ]
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(280px, 1fr))"
            , style "gap" "1rem"
            ]
            [ dashboardPage "Dashboard" "/" "Queue status, semantic indexer controls, system health overview"
            , dashboardPage "Messages" "/messages" "Browse conversation history by chat, view media attachments"
            , dashboardPage "Logs" "/logs" "Real-time structured event log with search and filtering"
            , dashboardPage "Jobs" "/jobs" "View and manage scheduled cron jobs"
            , dashboardPage "Settings" "/settings" "Configure message display options"
            , dashboardPage "Capabilities" "/capabilities" "This page - full feature reference"
            ]
        ]


dashboardPage : String -> String -> String -> Html msg
dashboardPage name path description =
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        , style "border-left" ("3px solid " ++ UI.colors.accent)
        ]
        [ div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.75rem"
            , style "margin-bottom" "0.75rem"
            ]
            [ span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.9375rem"
                , style "font-weight" "600"
                , style "color" UI.colors.textPrimary
                ]
                [ text name ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.6875rem"
                , style "color" UI.colors.textMuted
                , style "padding" "0.125rem 0.5rem"
                , style "background-color" UI.colors.bgTertiary
                , style "border-radius" "2px"
                ]
                [ text path ]
            ]
        , p
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            , style "margin" "0"
            , style "line-height" "1.5"
            ]
            [ text description ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- SEMANTIC SEARCH SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewSemanticSection : Html msg
viewSemanticSection =
    sectionCard "Semantic Search Indexer" "semantic"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "Background indexer that embeds memory files and Claude Code conversation history into a local vector database for semantic search. Toggle on/off from the Dashboard." ]
        , subsectionHeader "Data Sources"
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(280px, 1fr))"
            , style "gap" "1rem"
            , style "margin-bottom" "2rem"
            ]
            [ featureItem "Memory Files" "Watches the memory directory for .md file changes. Indexed immediately via filesystem watcher with debouncing." UI.colors.accent
            , featureItem "Conversations" "Polls Claude Code conversation .jsonl files every 5 minutes. Automatically cleans up orphaned subagent sessions." UI.colors.success
            ]
        , subsectionHeader "How It Works"
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(220px, 1fr))"
            , style "gap" "1rem"
            , style "margin-bottom" "2rem"
            ]
            [ semanticStep "1" "Chunk" "Documents split into overlapping chunks with heading context preserved"
            , semanticStep "2" "Embed" "Chunks embedded locally using FastEmbed (all-MiniLM-L6-v2, 384 dimensions)"
            , semanticStep "3" "Store" "Vectors stored in SQLite with SHA256 content hashing for incremental updates"
            , semanticStep "4" "Search" "Brute-force cosine similarity search (fast enough for <100k vectors)"
            ]
        , subsectionHeader "Dashboard Controls"
        , p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1rem 0"
            , style "font-size" "0.875rem"
            ]
            [ text "The Dashboard shows indexer status per data source including files indexed, stale count, and current activity. Use "
            , codeInline "Run Now"
            , text " to trigger an immediate conversation poll instead of waiting for the 5-minute interval."
            ]
        , subsectionHeader "Resource Limits"
        , configTable
            [ ( "Batch size", "Max chunks per embedding batch", "16" )
            , ( "Batch delay", "Pause between batches", "100ms" )
            , ( "Poll interval", "Conversation poll frequency", "5 minutes" )
            , ( "Memory watcher", "File change debounce time", "500ms" )
            ]
        ]


semanticStep : String -> String -> String -> Html msg
semanticStep number title description =
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        ]
        [ div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.75rem"
            , style "margin-bottom" "0.75rem"
            ]
            [ span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.6875rem"
                , style "font-weight" "700"
                , style "color" UI.colors.accent
                , style "padding" "0.25rem 0.5rem"
                , style "background-color" UI.colors.accentDim
                , style "border" ("1px solid " ++ UI.colors.accent)
                , style "border-radius" "2px"
                ]
                [ text number ]
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" UI.colors.textPrimary
                ]
                [ text title ]
            ]
        , p
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            , style "margin" "0"
            , style "line-height" "1.5"
            ]
            [ text description ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- CRON SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewCronSection : Html msg
viewCronSection =
    sectionCard "Cron Scheduling" "schedule"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "Schedule prompts to run at specific times. Supports one-shot delays and recurring jobs with cron expressions. Managed via MCP tools and visible in the Jobs dashboard page." ]
        , subsectionHeader "Job Types"
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(280px, 1fr))"
            , style "gap" "1rem"
            , style "margin-bottom" "2rem"
            ]
            [ featureItem "One-Shot" "Run a prompt once after a delay. Specify minutes from now." UI.colors.accent
            , featureItem "Recurring" "Run on a schedule using cron expressions. Keeps firing until cancelled." UI.colors.success
            , featureItem "Deferrable" "Jobs can be marked deferrable (default) so they wait for idle periods instead of interrupting active work." UI.colors.warning
            ]
        , subsectionHeader "Cron Expression Format"
        , codeBlock """# Standard 6-field cron format (with seconds)
# ┌─────── second (0-59)
# │ ┌───── minute (0-59)
# │ │ ┌─── hour (0-23)
# │ │ │ ┌─ day of month (1-31)
# │ │ │ │ ┌ month (1-12 or JAN-DEC)
# │ │ │ │ │ ┌ day of week (0-6 or SUN-SAT)
# │ │ │ │ │ │
  0 0 9 * * Mon    # Every Monday at 9:00 AM
  0 */30 * * * *   # Every 30 minutes
  0 0 8,12,18 * * * # At 8 AM, noon, and 6 PM daily"""
        ]


toolCard : String -> String -> List (Html msg) -> Html msg
toolCard name description params =
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "overflow" "hidden"
        ]
        [ div
            [ style "padding" "1rem 1.25rem"
            , style "background-color" UI.colors.bgTertiary
            , style "border-bottom" ("1px solid " ++ UI.colors.border)
            ]
            [ div
                [ style "font-family" UI.fontMono
                , style "font-size" "0.9375rem"
                , style "font-weight" "600"
                , style "color" UI.colors.accent
                , style "margin-bottom" "0.25rem"
                ]
                [ text name ]
            , div
                [ style "font-size" "0.8125rem"
                , style "color" UI.colors.textSecondary
                ]
                [ text description ]
            ]
        , div
            [ style "padding" "1rem 1.25rem"
            ]
            params
        ]


paramRow : String -> String -> String -> Html msg
paramRow name paramType description =
    div
        [ style "display" "flex"
        , style "align-items" "baseline"
        , style "gap" "0.75rem"
        , style "padding" "0.5rem 0"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        ]
        [ span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.8125rem"
            , style "font-weight" "500"
            , style "color" UI.colors.textPrimary
            , style "min-width" "120px"
            ]
            [ text name ]
        , span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.6875rem"
            , style "color" UI.colors.warning
            , style "padding" "0.125rem 0.375rem"
            , style "background-color" UI.colors.warningDim
            , style "border-radius" "2px"
            ]
            [ text paramType ]
        , span
            [ style "font-size" "0.75rem"
            , style "color" UI.colors.textMuted
            , style "flex" "1"
            ]
            [ text description ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- MCP SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewMcpSection : Html msg
viewMcpSection =
    sectionCard "MCP Integration" "mcp"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "Twolebot exposes a unified MCP server at "
            , codeInline "/mcp"
            , text " with all tools available through a single registration:"
            ]
        , codeBlock "claude mcp add twolebot --transport http http://localhost:8080/mcp"
        , subsectionHeader "Scheduling Tools"
        , p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1rem 0"
            , style "font-size" "0.875rem"
            ]
            [ text "Schedule Claude prompts to run at specific times or on recurring schedules." ]
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(300px, 1fr))"
            , style "gap" "1rem"
            , style "margin-bottom" "2rem"
            ]
            [ toolCard "schedule_job" "Schedule a new job"
                [ paramRow "prompt" "string" "The prompt to execute"
                , paramRow "in_minutes" "int?" "Minutes from now (one-shot)"
                , paramRow "cron" "string?" "Cron expression (recurring)"
                , paramRow "name" "string?" "Human-readable name"
                , paramRow "deferrable" "bool" "Wait for idle (default: true)"
                ]
            , toolCard "list_jobs" "List scheduled jobs"
                [ paramRow "status" "string?" "Filter: active, paused, or all"
                ]
            , toolCard "cancel_job" "Cancel a scheduled job"
                [ paramRow "job_id" "string" "The job ID to cancel"
                ]
            , toolCard "snooze_job" "Delay a job's next run"
                [ paramRow "job_id" "string" "The job ID to snooze"
                , paramRow "minutes" "int" "Minutes to delay"
                ]
            ]
        , subsectionHeader "Memory Tools"
        , p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1rem 0"
            , style "font-size" "0.875rem"
            ]
            [ text "Persistent markdown storage for notes, context, and information Claude should remember." ]
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(300px, 1fr))"
            , style "gap" "1rem"
            , style "margin-bottom" "2rem"
            ]
            [ toolCard "memory_read" "Read a memory file"
                [ paramRow "path" "string" "Relative path from memory directory"
                ]
            , toolCard "memory_write" "Write or append to a memory file"
                [ paramRow "path" "string" "Relative path (must end in .md)"
                , paramRow "content" "string" "Content to write"
                , paramRow "mode" "string?" "\"replace\" (default) or \"append\""
                ]
            , toolCard "memory_search" "Search across memory files"
                [ paramRow "query" "string" "Regex pattern to search for"
                , paramRow "limit" "int?" "Max results (default: 10)"
                ]
            ]
        , subsectionHeader "Conversation Tools"
        , p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1rem 0"
            , style "font-size" "0.875rem"
            ]
            [ text "Search Claude Code conversation history across all projects." ]
        , div
            [ style "display" "grid"
            , style "grid-template-columns" "repeat(auto-fit, minmax(300px, 1fr))"
            , style "gap" "1rem"
            ]
            [ toolCard "conversation_search" "Search conversation history"
                [ paramRow "query" "string" "Regex pattern to search for"
                , paramRow "project" "string?" "Filter to specific project"
                , paramRow "limit" "int?" "Max results (default: 10)"
                , paramRow "context_before" "int?" "Messages before match (default: 3)"
                , paramRow "context_after" "int?" "Messages after match (default: 3)"
                , paramRow "recency_weight" "float?" "0.0-1.0, higher = prefer recent"
                ]
            ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- CONFIG SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewConfigSection : Html msg
viewConfigSection =
    sectionCard "Configuration" "config"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "Configuration priority: CLI arguments > Environment variables > config.toml file" ]
        , subsectionHeader "Environment Variables"
        , div
            [ style "margin-bottom" "2rem"
            ]
            [ configTable
                [ ( "TELEGRAM_BOT_TOKEN", "Bot token from @BotFather", "Required" )
                , ( "GEMINI_API_KEY", "Google AI Studio API key", "Required" )
                ]
            ]
        , subsectionHeader "CLI Arguments"
        , codeBlock """twolebot \\
  --telegram-token "BOT_TOKEN" \\
  --gemini-key "API_KEY" \\
  --data-dir ./data \\
  --port 8080 \\
  --claude-model "claude-opus-4-6" \\
  --process-timeout-ms 600000 \\
  --typing-interval-secs 6 \\
  --cron-idle-threshold-secs 600"""
        , subsectionHeader "Configuration File"
        , p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1rem 0"
            , style "font-size" "0.875rem"
            ]
            [ text "Located at "
            , codeInline "~/.config/twolebot/config.toml"
            , text " (Linux) or "
            , codeInline "~/Library/Application Support/twolebot/config.toml"
            , text " (macOS)"
            ]
        , codeBlock """telegram_token = "your_bot_token"
gemini_key = "your_gemini_key"
claude_model = "claude-opus-4-6"
port = 8080
process_timeout_ms = 600000
typing_interval_secs = 4
cron_idle_threshold_secs = 600"""
        , subsectionHeader "All Options"
        , configTable
            [ ( "--port", "HTTP server port", "8080" )
            , ( "--data-dir", "Data directory path", "XDG data dir" )
            , ( "--config-dir", "Config directory path", "XDG config dir" )
            , ( "--memory-dir", "Memory storage path", "{data-dir}/memory" )
            , ( "--claude-model", "Claude model to use", "claude-opus-4-6" )
            , ( "--process-timeout-ms", "Max Claude process time", "600000 (10 min)" )
            , ( "--typing-interval-secs", "Telegram typing indicator interval", "4" )
            , ( "--cron-idle-threshold-secs", "Idle time before promoting cron jobs", "600 (10 min)" )
            ]
        ]


configTable : List ( String, String, String ) -> Html msg
configTable rows =
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "overflow" "hidden"
        , style "margin-bottom" "1.5rem"
        ]
        [ div
            [ style "display" "grid"
            , style "grid-template-columns" "minmax(180px, 1fr) 2fr 1fr"
            , style "gap" "0"
            , style "padding" "0.75rem 1rem"
            , style "background-color" UI.colors.bgTertiary
            , style "border-bottom" ("1px solid " ++ UI.colors.border)
            ]
            [ tableHeaderCell "Option"
            , tableHeaderCell "Description"
            , tableHeaderCell "Default"
            ]
        , div [] (List.map configRow rows)
        ]


configRow : ( String, String, String ) -> Html msg
configRow ( option, description, defaultVal ) =
    div
        [ style "display" "grid"
        , style "grid-template-columns" "minmax(180px, 1fr) 2fr 1fr"
        , style "gap" "0"
        , style "padding" "0.75rem 1rem"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        ]
        [ span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.8125rem"
            , style "color" UI.colors.accent
            ]
            [ text option ]
        , span
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            ]
            [ text description ]
        , span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.75rem"
            , style "color" UI.colors.textMuted
            ]
            [ text defaultVal ]
        ]


tableHeaderCell : String -> Html msg
tableHeaderCell label =
    span
        [ style "font-family" UI.fontMono
        , style "font-size" "0.625rem"
        , style "font-weight" "700"
        , style "text-transform" "uppercase"
        , style "letter-spacing" "0.1em"
        , style "color" UI.colors.textMuted
        ]
        [ text label ]


-- ═══════════════════════════════════════════════════════════════════════════
-- STORAGE SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewStorageSection : Html msg
viewStorageSection =
    sectionCard "Data Storage" "storage"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "All data is stored locally on the filesystem. No cloud dependencies." ]
        , subsectionHeader "Linux (XDG)"
        , codeBlock """~/.config/twolebot/config.toml      # Configuration (secrets)
~/.local/share/twolebot/            # Data directory
~/.local/state/twolebot/            # Logs"""
        , subsectionHeader "macOS"
        , codeBlock """~/Library/Application Support/twolebot/config.toml  # Config
~/Library/Application Support/twolebot/             # Data"""
        , subsectionHeader "Data Directory Structure"
        , codeBlock """data/
├── prompts/
│   ├── pending/          # Inbound queue
│   ├── running/          # Currently processing
│   └── completed/        # Finished prompts
├── responses/
│   ├── pending/          # Outbound queue
│   ├── sent/             # Delivered
│   └── failed/           # Failed delivery
├── cron/
│   ├── jobs/             # Job definitions
│   └── executions/       # Run history
├── messages/{chatId}/    # Message history
├── media/{chatId}/       # Downloaded files
├── memory/               # MCP memory files
├── vectors.db            # Semantic search embeddings (SQLite)
└── logs.jsonl            # Event log (10k entries)"""
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- API SECTION
-- ═══════════════════════════════════════════════════════════════════════════


viewApiSection : Html msg
viewApiSection =
    sectionCard "REST API" "api"
        [ p
            [ style "color" UI.colors.textSecondary
            , style "margin" "0 0 1.5rem 0"
            ]
            [ text "HTTP API for programmatic access. All endpoints return JSON." ]
        , apiTable
            [ ( "GET", "/api/status", "System health and version" )
            , ( "GET", "/api/feed", "Prompt queue status" )
            , ( "GET", "/api/responses", "Response queue status" )
            , ( "GET", "/api/messages", "List all chats" )
            , ( "GET", "/api/messages/{chatId}", "Chat history (paginated)" )
            , ( "GET", "/api/media/{chatId}/{file}", "Media file download" )
            , ( "GET", "/api/logs", "Event log (paginated, searchable)" )
            , ( "GET", "/api/cron/jobs", "List cron jobs" )
            , ( "POST", "/api/cron/jobs", "Create cron job" )
            , ( "DELETE", "/api/cron/jobs/{id}", "Cancel cron job" )
            , ( "POST", "/api/cron/jobs/{id}/pause", "Pause job" )
            , ( "POST", "/api/cron/jobs/{id}/resume", "Resume job" )
            , ( "GET", "/api/settings", "Current settings" )
            , ( "PUT", "/api/settings", "Update settings" )
            , ( "GET", "/api/semantic/status", "Semantic indexer status and stats" )
            , ( "POST", "/api/semantic/toggle", "Enable/disable semantic indexer" )
            , ( "POST", "/api/semantic/reindex", "Trigger immediate conversation reindex" )
            , ( "GET", "/api/setup/status", "Setup wizard status" )
            , ( "*", "/mcp", "MCP server (all tools)" )
            ]
        ]


apiTable : List ( String, String, String ) -> Html msg
apiTable rows =
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "overflow" "hidden"
        ]
        [ div
            [ style "display" "grid"
            , style "grid-template-columns" "80px 1fr 1fr"
            , style "gap" "0"
            , style "padding" "0.75rem 1rem"
            , style "background-color" UI.colors.bgTertiary
            , style "border-bottom" ("1px solid " ++ UI.colors.border)
            ]
            [ tableHeaderCell "Method"
            , tableHeaderCell "Endpoint"
            , tableHeaderCell "Description"
            ]
        , div [] (List.map apiRow rows)
        ]


apiRow : ( String, String, String ) -> Html msg
apiRow ( method, endpoint, description ) =
    let
        methodColor =
            case method of
                "GET" -> UI.colors.success
                "POST" -> UI.colors.warning
                "PUT" -> UI.colors.accent
                "DELETE" -> UI.colors.error
                _ -> UI.colors.textMuted
    in
    div
        [ style "display" "grid"
        , style "grid-template-columns" "80px 1fr 1fr"
        , style "gap" "0"
        , style "padding" "0.625rem 1rem"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        ]
        [ span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.6875rem"
            , style "font-weight" "700"
            , style "color" methodColor
            ]
            [ text method ]
        , span
            [ style "font-family" UI.fontMono
            , style "font-size" "0.8125rem"
            , style "color" UI.colors.textPrimary
            ]
            [ text endpoint ]
        , span
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            ]
            [ text description ]
        ]


-- ═══════════════════════════════════════════════════════════════════════════
-- SHARED COMPONENTS
-- ═══════════════════════════════════════════════════════════════════════════


sectionCard : String -> String -> List (Html msg) -> Html msg
sectionCard title sectionId content =
    div
        [ id sectionId
        , style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "6px"
        , style "padding" "2rem"
        , style "position" "relative"
        ]
        [ -- Section number/accent
          div
            [ style "position" "absolute"
            , style "top" "0"
            , style "left" "0"
            , style "width" "4px"
            , style "height" "100%"
            , style "background" ("linear-gradient(180deg, " ++ UI.colors.accent ++ " 0%, transparent 100%)")
            , style "border-radius" "6px 0 0 6px"
            ]
            []
        , h3
            [ style "font-family" UI.fontDisplay
            , style "font-size" "1.375rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.02em"
            , style "color" UI.colors.textPrimary
            , style "margin" "0 0 1.25rem 0"
            , style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.75rem"
            ]
            [ div
                [ style "width" "8px"
                , style "height" "8px"
                , style "background-color" UI.colors.accent
                , style "box-shadow" ("0 0 12px " ++ UI.colors.accent)
                ]
                []
            , text title
            ]
        , div [] content
        ]


subsectionHeader : String -> Html msg
subsectionHeader title =
    h4
        [ style "font-family" UI.fontMono
        , style "font-size" "0.75rem"
        , style "font-weight" "600"
        , style "letter-spacing" "0.1em"
        , style "text-transform" "uppercase"
        , style "color" UI.colors.textMuted
        , style "margin" "1.5rem 0 1rem 0"
        , style "padding-bottom" "0.5rem"
        , style "border-bottom" ("1px solid " ++ UI.colors.border)
        ]
        [ text title ]


featureItem : String -> String -> String -> Html msg
featureItem title description accentColor =
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        , style "border-left" ("3px solid " ++ accentColor)
        ]
        [ div
            [ style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.625rem"
            , style "margin-bottom" "0.625rem"
            ]
            [ div
                [ style "width" "6px"
                , style "height" "6px"
                , style "background-color" accentColor
                , style "border-radius" "1px"
                , style "box-shadow" ("0 0 8px " ++ accentColor)
                ]
                []
            , span
                [ style "font-family" UI.fontMono
                , style "font-size" "0.875rem"
                , style "font-weight" "600"
                , style "color" UI.colors.textPrimary
                ]
                [ text title ]
            ]
        , p
            [ style "font-size" "0.8125rem"
            , style "color" UI.colors.textSecondary
            , style "margin" "0"
            , style "line-height" "1.5"
            ]
            [ text description ]
        ]


codeInline : String -> Html msg
codeInline code =
    span
        [ style "font-family" UI.fontMono
        , style "font-size" "0.8125rem"
        , style "background-color" UI.colors.bgSurface
        , style "padding" "0.125rem 0.5rem"
        , style "border-radius" "2px"
        , style "color" UI.colors.accent
        ]
        [ text code ]


codeBlock : String -> Html msg
codeBlock code =
    pre
        [ style "font-family" UI.fontMono
        , style "font-size" "0.8125rem"
        , style "line-height" "1.6"
        , style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1.25rem"
        , style "margin" "0 0 1.5rem 0"
        , style "overflow-x" "auto"
        , style "color" UI.colors.textSecondary
        ]
        [ text code ]
