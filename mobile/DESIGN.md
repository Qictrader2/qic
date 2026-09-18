# QICTRADER Design System

> Source of truth for the QICTRADER visual identity. All UI work must reference this document.
> Generated from codebase extraction — April 2026.

---

## 1. Brand Identity

- **Brand Name:** QICTRADER (one word, all caps in marketing; "QicTrader" in code/UI)
- **Tagline:** South Africa's P2P Crypto Marketplace
- **Primary Brand Color:** `#00A3F6` (Cyan Blue)
- **PWA Theme Color:** `#00A3F6`
- **Default Theme:** System preference (dark/light)
- **Personality:** Trustworthy, modern, accessible, crypto-native

---

## 2. Color System

### 2.1 Brand Colors

| Token | Hex | Role |
|-------|-----|------|
| `--brand-blue` | `#00A3F6` | Primary action, CTAs, links, focus rings |
| `--brand-blue-light` | `#38BDF8` | Hover accents, highlighted text |
| `--brand-blue-dark` | `#0284C7` | Pressed/active states |
| `--brand-blue-bg` | `rgba(0,163,246,0.1)` | Subtle brand backgrounds |
| `--brand-green` | `#10B981` | Success, positive indicators |
| `--brand-green-dark` | `#059669` | Success pressed state |
| `--brand-red` | `#EF4444` | Error, destructive actions |
| `--brand-red-dark` | `#DC2626` | Error pressed state |

### 2.2 Semantic Colors

| Token | Light | Dark | Role |
|-------|-------|------|------|
| `--success` | `#10B981` | `#34D399` | Positive outcomes |
| `--success-bg` | `rgba(16,185,129,0.1)` | `rgba(52,211,153,0.15)` | Success backgrounds |
| `--warning` | `#F59E0B` | `#FBBF24` | Caution states |
| `--warning-bg` | `rgba(245,158,11,0.1)` | `rgba(251,191,36,0.15)` | Warning backgrounds |
| `--error` | `#EF4444` | `#F87171` | Error states |
| `--error-bg` | `rgba(239,68,68,0.1)` | `rgba(248,113,113,0.15)` | Error backgrounds |
| `--info` | `#3B82F6` | `#60A5FA` | Informational |
| `--info-bg` | `rgba(59,130,246,0.1)` | `rgba(96,165,250,0.15)` | Info backgrounds |

### 2.3 Surface Colors

| Token | Light | Dark |
|-------|-------|------|
| `--background` | `#FFFFFF` | `#000000` |
| `--background-secondary` | `#FFFFFF` | `#111111` |
| `--background-gray` | `#F6F6F6` | `#040607` |
| `--foreground` | `#0F172A` | `#F8FAFC` |
| `--card` | `#FFFFFF` | `#191F2A` |
| `--card-elevated` | `#FFFFFF` | `#334155` |
| `--surface` | `#F8FAFC` | `#1E293B` |
| `--surface-hover` | `#F1F5F9` | `#334155` |
| `--surface-dark` | `#E2E8F0` | `#0F172A` |
| `--popover` | `#FFFFFF` | `#1E293B` |

### 2.4 Text Colors

| Token | Light | Dark |
|-------|-------|------|
| `--foreground` | `#0F172A` | `#F8FAFC` |
| `--muted-foreground` | `#475569` | `#94A3B8` |
| `--text-secondary` | `#64748B` | `#94A3B8` |
| `--text-tertiary` | `#94A3B8` | `#64748B` |
| `--text-placeholder` | `#9CA3AF` | `#64748B` |

### 2.5 Border Colors

| Token | Light | Dark |
|-------|-------|------|
| `--border` | `#E2E8F0` | `rgba(255,255,255,0.1)` |
| `--border-subtle` | `rgba(0,0,0,0.08)` | `rgba(255,255,255,0.1)` |
| `--input` | `#E2E8F0` | `rgba(255,255,255,0.15)` |
| `--ring` | `#94A3B8` | `#64748B` |

### 2.6 Chart Colors

| Token | Light | Dark | Use |
|-------|-------|------|-----|
| `--chart-1` | `#3B82F6` | `#60A5FA` | Primary series |
| `--chart-2` | `#10B981` | `#34D399` | Secondary series |
| `--chart-3` | `#F59E0B` | `#FBBF24` | Tertiary series |
| `--chart-4` | `#8B5CF6` | `#A78BFA` | Quaternary series |
| `--chart-5` | `#EC4899` | `#F472B6` | Quinary series |

### 2.7 Cryptocurrency Colors

| Coin | Color | Hex |
|------|-------|-----|
| BTC | Orange | `#F7931A` |
| ETH | Indigo | `#627EEA` |
| SOL | Violet | `#9945FF` |
| USDT | Teal | `#26A17B` |

### 2.8 Affiliate Tier Colors

| Tier | Primary | Accent | Glow |
|------|---------|--------|------|
| Novice | `#6B7280` | `#9CA3AF` | None |
| Bronze | `#CD7F32` | `#E8A858` | `rgba(205,127,50,0.3)` |
| Silver | `#C0C0C0` | `#E0E0E0` | `rgba(192,192,192,0.3)` |
| Gold | `#C9A84C` | `#F0D070` | `rgba(201,168,76,0.35)` |
| Diamond | `#B9F2FF` | `#7DF3FF` | `rgba(185,242,255,0.4)` |

---

## 3. Typography

### 3.1 Font Family

| Role | Font | Fallback |
|------|------|----------|
| Sans (primary) | **Poppins** | system-ui, sans-serif |
| Mono | Poppins | (no dedicated monospace font) |

**Loaded weights:** 400 (Regular), 500 (Medium), 600 (SemiBold), 700 (Bold)

CSS variable: `--font-poppins` → `--font-sans`

### 3.2 Type Scale

| Name | Size | Tailwind | Primary Use |
|------|------|----------|-------------|
| Micro | 8–9px | `text-[8px]` / `text-[9px]` | Badges, micro labels |
| Tiny | 10–11px | `text-[10px]` / `text-[11px]` | Payment labels, premium badges |
| XS | 12px | `text-xs` | Table headers, helper text, badges |
| SM | 14px | `text-sm` | **Default body text**, labels, buttons |
| Base | 16px | `text-base` | Inputs, field legends |
| LG | 18px | `text-lg` | Dialog titles, subheadings |
| XL | 20px | `text-xl` | Section headings |
| 2XL | 24px | `text-2xl` | Card titles, page headings |
| 3XL | 30px | `text-3xl` | Page headings (desktop) |
| 4XL | 36px | `text-4xl` | Hero heading (tablet) |
| 5XL | 48px | `text-5xl` | Hero heading (desktop) |

### 3.3 Font Weight Usage

| Weight | Tailwind | Frequency | Use |
|--------|----------|-----------|-----|
| 400 | `font-normal` | Low | Body text, descriptions |
| **500** | **`font-medium`** | **Highest** | Labels, buttons, form fields |
| 600 | `font-semibold` | High | Card titles, headings, emphasis |
| 700 | `font-bold` | Medium | Hero headings, strong emphasis |

### 3.4 Heading Hierarchy

| Level | Mobile | Desktop | Weight | Line Height |
|-------|--------|---------|--------|-------------|
| H1 (Hero) | `text-3xl` | `text-5xl` | `font-medium` | `leading-tight` |
| H1 (Page) | `text-2xl` | `text-3xl` | `font-semibold` | `leading-none` |
| H2 | `text-xl` | `text-2xl` | `font-bold` | `leading-none` |
| H3 | `text-base` | `text-lg` | `font-semibold` | `leading-none` |
| H4 | `text-sm` | `text-base` | `font-semibold` | `leading-none` |

### 3.5 Line Height

| Token | Value | Use |
|-------|-------|-----|
| `leading-none` | 1.0 | Headings, card titles, badges |
| `leading-tight` | 1.25 | Compact heading text |
| `leading-snug` | 1.375 | Form labels |
| `leading-normal` | 1.5 | Standard body |
| `leading-relaxed` | 1.625 | Readable paragraphs |

### 3.6 Letter Spacing

| Token | Value | Use |
|-------|-------|-----|
| `tracking-tight` | -0.025em | Card/dialog titles |
| `tracking-normal` | 0 | Body text (default) |
| `tracking-wider` | 0.05em | Table headers, uppercase labels |
| `tracking-widest` | 0.1em | Crypto addresses |

---

## 4. Spacing

### 4.1 Spacing Scale (by frequency of use)

| Token | Value | Use |
|-------|-------|-----|
| `gap-1` | 4px | Tight inline spacing |
| `gap-2` | 8px | **Default gap** — flex/grid items |
| `gap-3` | 12px | Secondary gap |
| `gap-4` | 16px | Larger section gaps |
| `gap-6` | 24px | Section-level gaps |
| `p-3` | 12px | Medium padding |
| `p-4` | 16px | **Default padding** — cards, containers |
| `p-6` | 24px | Card header/content padding |
| `p-8` | 32px | Large container padding |
| `px-3` | 12px | Horizontal padding — inputs, badges |
| `px-4` | 16px | Horizontal padding — buttons |
| `px-6` | 24px | Horizontal padding — CTAs |
| `py-1` | 4px | Tight vertical padding |
| `py-2` | 8px | Button vertical padding |
| `py-3` | 12px | Header vertical padding |

### 4.2 Component-Specific Spacing

| Component | Padding | Gap |
|-----------|---------|-----|
| Card Header | `p-6` | `space-y-1.5` |
| Card Content | `p-6 pt-0` | — |
| Modal (mobile) | `p-4` | — |
| Modal (desktop) | `p-6` | — |
| Button (default) | `px-4 py-2` | — |
| Button (lg) | `px-6 py-4` or `px-8 py-4` | — |
| Input | `px-3 py-2` | — |
| Badge | `px-2.5 py-0.5` | — |
| Header (desktop) | `px-24 py-3` | `gap-2` |
| Header (mobile) | `px-3 py-2.5` | `gap-1.5` |
| Field Group | — | `gap-7` |

---

## 5. Border Radius

| Token | Value | Use | Frequency |
|-------|-------|-----|-----------|
| `--radius` | `0.625rem` (10px) | Base radius | — |
| `rounded-sm` | 4px | Small elements | Low |
| `rounded-md` | 6px | Buttons, inputs | Medium |
| `rounded-lg` | 8px | **Cards, containers** | **Highest** |
| `rounded-xl` | 12px | Prominent elements, modals | High |
| `rounded-2xl` | 16px | Mobile modal tops | Low |
| `rounded-full` | 9999px | Avatars, badges, pills | High |

---

## 6. Elevation & Shadows

| Level | Shadow | Use |
|-------|--------|-----|
| 0 | None | Flat elements |
| 1 | `shadow-sm` | Inputs, subtle elevation |
| 2 | `shadow-md` | Dropdowns, menus |
| 3 | `shadow-lg` | Hover cards, dropdown content |
| 4 | `shadow-xl` | Modals |
| 5 | `shadow-2xl` | Large emphasized modals |
| Glow | `shadow-[0_0_10px_rgba(0,163,246,0.7)]` | Brand blue glow effect |
| Dark | `shadow-black/20` to `shadow-black/40` | Dark overlays |

### Backdrop

- Modal backdrop: `bg-black/60 backdrop-blur-sm`
- Overlay: `rgba(0,0,0,0.5)` (light) / `rgba(0,0,0,0.75)` (dark)

---

## 7. Component Patterns

### 7.1 Buttons

| Variant | Background | Text | Border |
|---------|-----------|------|--------|
| **Default** | `bg-brand-blue` | `text-white` | None |
| Destructive | `bg-destructive` | `text-white` | None |
| Outline | `bg-background` | `text-foreground` | `border-input` |
| Secondary | `bg-secondary` | `text-secondary-foreground` | None |
| Ghost | Transparent | `text-foreground` | None |
| Link | Transparent | `text-primary` | None |

**Sizes:** `sm` (h-9), `default` (h-10), `lg` (h-11), `icon` (h-10 w-10)

**States:**
- Hover: 90% opacity on primary, `bg-accent` on ghost/outline
- Focus: `ring-2 ring-offset-2`
- Disabled: `opacity-50 pointer-events-none`
- Active: `scale(0.98) opacity-0.9`

### 7.2 Cards

```
Base: bg-card rounded-lg border shadow-sm
Header: p-6, gap space-y-1.5
Content: p-6 pt-0
Footer: p-6 pt-0, flex items-center
Hover: border-muted-foreground/30, shadow-lg
```

### 7.3 Inputs

```
Base: h-10 rounded-md border px-3 py-2 text-sm
Background: bg-background/20
Border: border-input
Focus: ring-2 ring-ring ring-offset-2
Disabled: opacity-50 cursor-not-allowed
```

### 7.4 Modals

```
Mobile: rounded-t-2xl, slide-in-from-bottom, max-h-[90vh]
Desktop: rounded-2xl, zoom-in-95, max-h-[85vh]
Backdrop: bg-black/60 backdrop-blur-sm
Sizes: sm (384px), md (448px), lg (512px), xl (576px)
```

### 7.5 Badges

| Variant | Background | Text |
|---------|-----------|------|
| Default | `bg-primary` | `text-primary-foreground` |
| Secondary | `bg-secondary` | `text-secondary-foreground` |
| Destructive | `bg-destructive` | `text-white` |
| Outline | Transparent | `text-foreground` |
| Success | `bg-success-bg` | `text-success` |
| Warning | `bg-warning-bg` | `text-warning` |

Base: `rounded-md px-2.5 py-0.5 text-xs font-semibold`

### 7.6 Tables

```
Wrapper: rounded-lg border overflow-auto
Header: bg-gray-900/50
Head Cell: h-12 px-6 py-4 text-xs font-semibold tracking-wider uppercase text-gray-400
Body: divide-y divide-gray-800
Row Hover: hover:bg-primary-foreground
Cell: text-sm
```

### 7.7 Switches (Hairline)

The canonical toggle. Used in `NotificationSettings`, `SecuritySettings`, and any
ToggleItem row. Source: `frontend/src/components/ui/switch.tsx`.

| Property | Off | On |
|----------|-----|----|
| Track size | 44 × 24 px | 44 × 24 px |
| Track radius | `rounded-full` | `rounded-full` |
| Track background | `transparent` | `brand-green` (`#10b981`) |
| Track border | `1px white/10` | `1px brand-green` |
| Thumb size | 16 × 16 px | 16 × 16 px |
| Thumb color | `slate-400` (`#94a3b8`) | `white` |
| Thumb position | `translate-x-[3px]` | `translate-x-[23px]` |
| Transition | 250ms ease-out (track), 250ms cubic-bezier(0.4,0,0.2,1) (thumb) | same |
| Focus ring | `focus-visible:ring-[3px] ring-brand-green/15` | same |

**Required class**: `no-min-touch`. The global `button:not(.no-min-touch)` rule
in `globals.css` forces `min-height: 44px` on all buttons for tap-target
accessibility. Without `no-min-touch` the 24px-tall track expands to a 44×44
square — a regression we shipped once. Keep the class on the Switch primitive.

Touch-target compensation: the 44px width still meets WCAG 2.5.5 on the
dominant axis, and the row container (`py-3`) gives an effective vertical hit
area of ~48px around the toggle.

---

## 8. Animation & Motion

### 8.1 Entry Animations

| Animation | Duration | Easing | Use |
|-----------|----------|--------|-----|
| `fade-in` | 200ms | ease-out | General appearance |
| `slide-in-from-top` | 200ms | ease-out | Dropdowns |
| `slide-in-from-bottom` | 200ms | ease-out | Mobile modals |
| `zoom-in-95` | 200ms | ease-out | Desktop modals |

### 8.2 Transitions

| Property | Duration | Use |
|----------|----------|-----|
| `transition-colors` | 150ms | Hover color changes |
| `transition-all` | 150ms | Multi-property changes |
| `transition-transform` | 150ms | Scale/translate |
| `duration-200` | 200ms | Focus states |
| `duration-300` | 300ms | Modal animations |

### 8.3 Brand Animations

| Name | Duration | Use |
|------|----------|-----|
| `crypto-float` | 6s | Floating crypto elements |
| `brand-pulse` | 3s | Blue glow pulsing |
| `breathe` | 4s | Scale 1→1.02 breathing |
| `orbit` | 20s | Rotating orbital elements |
| `coin-spin` | 4s | 3D coin rotation |
| `scroll-left` | 30s | Marquee scrolling |
| `border-glow` | 3s | Pulsing border glow |
| `shimmer` | 1.5s | Skeleton loading |

### 8.4 Interaction Animations (Framer Motion)

| Element | Animation | Timing |
|---------|-----------|--------|
| Hero H1 | fade + translateY(30→0) | 0.7s, custom ease |
| Hero subtitle | fade + translateY(20→0) | 0.6s, 0.5s delay |
| Hero CTAs | fade + translateY(20→0) | 0.6s, 0.7s delay |
| Hero image | float up/down | 5s infinite |
| Background orbs | drift x/y | 6–8s infinite |
| ScaleOnHover | scale on hover | instant |
| SlideIn | translateX direction | 0.5s |

---

## 9. Layout

### 9.1 Breakpoints

| Name | Width | Use |
|------|-------|-----|
| `sm` | 640px | Mobile → tablet |
| `md` | 768px | Tablet adjustments |
| `lg` | 1024px | Desktop layout switch |
| `xl` | 1280px | Large desktop |

### 9.2 Container Widths

| Token | Width | Use |
|-------|-------|-----|
| `max-w-md` | 448px | Dialogs, small containers |
| `max-w-lg` | 512px | Modals |
| `max-w-xl` | 576px | Medium containers |
| `max-w-2xl` | 672px | Cards |
| `max-w-3xl` | 768px | Text content |
| `max-w-4xl` | 896px | Page sections |
| `max-w-6xl` | 1152px | Hero sections |
| `max-w-7xl` | 1280px | Full-width hero |

### 9.3 Grid Patterns

| Pattern | Use |
|---------|-----|
| `grid-cols-1 sm:grid-cols-2 lg:grid-cols-3` | Feature cards, testimonials |
| `grid-cols-1 md:grid-cols-2` | Two-column layouts |
| `grid-cols-2 gap-3` | Compact grids |

### 9.4 Z-Index Scale

| Value | Use |
|-------|-----|
| `z-10` | Modal inner content |
| `z-40` | Header, sticky nav |
| `z-50` | Modals, dropdowns, toasts |
| `z-[9999]` | Critical overlays (progress bar) |

---

## 10. Responsive Patterns

### Mobile-First Strategy

- Base styles target mobile
- Progressive enhancement via breakpoints
- Bottom-sheet modals on mobile, centered on desktop
- Single-column → multi-column grid transitions
- Touch targets: minimum 44×44px

### Key Breakpoint Behaviors

| Element | Mobile | Desktop |
|---------|--------|---------|
| Header padding | `px-3` | `px-24` |
| Navigation | Hamburger drawer | Inline links |
| Hero layout | Stacked (vertical) | Side-by-side |
| Feature grid | 1 column | 3–4 columns |
| Modal style | Bottom sheet | Centered dialog |
| Card grid | 1 column | 2–3 columns |

---

## 11. Anti-Patterns (Do NOT use)

- **Fonts:** Do not introduce Inter, Roboto, Arial, Space Grotesk, or system fonts. Poppins only.
- **Colors:** No purple-on-white gradients. No colors outside the defined palette without approval.
- **Radius:** No `rounded-3xl` or larger except `rounded-full` for pills/avatars.
- **Shadows:** No colored shadows except the brand blue glow.
- **Layout:** No centered hero + three-column feature grid (generic AI pattern).
- **Suppression:** No `let _ =` on Result types. No `.ok()` on financial paths.
- **Components:** No custom components when Shadcn equivalents exist.
- **Hardcoded values:** Use CSS variables/Tailwind tokens. No raw hex in components.

---

## 12. File Reference

| File | Contents |
|------|----------|
| `src/app/globals.css` | CSS variables, animations, global styles |
| `src/app/layout.tsx` | Font import (Poppins), metadata, root layout |
| `src/components/ui/` | Shadcn component library |
| `components.json` | Shadcn config (New York style) |
| `public/logo.svg` | Brand logo |
| `public/landing/` | Landing page assets |
| `public/icons/` | Favicons and app icons |

---

*This document is the single source of truth for QICTRADER's visual identity. Update it when design decisions change.*
