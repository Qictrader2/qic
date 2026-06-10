import {
  View,
  Text,
  ScrollView,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState, useEffect } from "react"
import AsyncStorage from "@react-native-async-storage/async-storage"
import {
  ChevronLeft,
  ChevronRight,
  ShieldCheck,
  CheckCircle2,
  Banknote,
  Settings2,
  Tag,
} from "lucide-react-native"
import { useCreateOffer } from "@/src/hooks/api/use-market"
import type { OfferType, Currency, PaymentMethodType } from "@/src/services/market.service"
import { ApiError } from "@/src/lib/api/client"
import { trackEvent } from "@/src/lib/analytics"

const DRAFT_KEY = "qic_create_offer_draft"

const CURRENCIES: Currency[] = ["BTC", "ETH", "SOL", "USDT", "USDC"]
const FIAT_OPTIONS = ["ZAR", "USD", "NGN"]
const PAYMENT_METHODS: { id: PaymentMethodType; label: string }[] = [
  { id: "bank_transfer", label: "Bank transfer" },
  { id: "cash", label: "Cash" },
  { id: "mobile_money", label: "Mobile money" },
  { id: "crypto", label: "Crypto" },
  { id: "other", label: "Other" },
]

const schema = z.object({
  offerType: z.enum(["buy", "sell"]),
  currency: z.enum(["BTC", "ETH", "SOL", "USDT", "USDC"]),
  fiatCurrency: z.string().min(1),
  pricePerUnit: z.string().refine((v) => parseFloat(v) > 0, "Enter a price"),
  minAmount: z.string().refine((v) => parseFloat(v) > 0, "Enter a minimum"),
  maxAmount: z.string().refine((v) => parseFloat(v) > 0, "Enter a maximum"),
  totalAmount: z.string().refine((v) => parseFloat(v) > 0, "Enter total available"),
  paymentMethods: z.array(z.string()).min(1, "Select at least one payment method"),
  paymentWindow: z.string().refine((v) => parseInt(v) >= 15, "Minimum 15 minutes"),
  terms: z.string().optional(),
})

type Form = z.infer<typeof schema>

const STEPS = [
  { num: 1, label: "Asset", icon: Tag },
  { num: 2, label: "Price", icon: Banknote },
  { num: 3, label: "Settings", icon: Settings2 },
  { num: 4, label: "Review", icon: CheckCircle2 },
] as const

export default function CreateOfferScreen() {
  const router = useRouter()
  const { mutateAsync: createOffer } = useCreateOffer()
  const [serverError, setServerError] = useState<string | null>(null)
  const [draftRestored, setDraftRestored] = useState(false)
  const [step, setStep] = useState(1)

  const {
    control,
    handleSubmit,
    watch,
    setValue,
    reset,
    trigger,
    formState: { errors, isSubmitting },
  } = useForm<Form>({
    resolver: zodResolver(schema),
    defaultValues: {
      offerType: "sell",
      currency: "USDT",
      fiatCurrency: "ZAR",
      paymentMethods: [],
      paymentWindow: "30",
    },
    mode: "onChange",
  })

  const formValues = watch()
  const { offerType, currency, fiatCurrency, paymentMethods: selectedPaymentMethods = [] } = formValues

  useEffect(() => {
    AsyncStorage.getItem(DRAFT_KEY).then((raw) => {
      if (!raw) return
      try {
        const draft = JSON.parse(raw) as Partial<Form>
        reset({ ...formValues, ...draft })
        setDraftRestored(true)
        setTimeout(() => setDraftRestored(false), 3000)
      } catch {}
    })
  }, [])

  useEffect(() => {
    const t = setTimeout(() => {
      AsyncStorage.setItem(DRAFT_KEY, JSON.stringify(formValues)).catch(() => {})
    }, 500)
    return () => clearTimeout(t)
  }, [JSON.stringify(formValues)])

  function togglePaymentMethod(id: string) {
    const current = selectedPaymentMethods
    if (current.includes(id)) {
      setValue("paymentMethods", current.filter((m) => m !== id))
    } else {
      setValue("paymentMethods", [...current, id])
    }
  }

  async function next() {
    const fieldsByStep: Record<number, (keyof Form)[]> = {
      1: ["offerType", "currency", "fiatCurrency"],
      2: ["pricePerUnit", "minAmount", "maxAmount", "totalAmount"],
      3: ["paymentMethods", "paymentWindow"],
    }
    const fields = fieldsByStep[step]
    if (fields) {
      const ok = await trigger(fields)
      if (!ok) return
    }
    if (step < 4) setStep(step + 1)
  }

  function back() {
    if (step > 1) setStep(step - 1)
    else router.back()
  }

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await createOffer({
        offerType: data.offerType as OfferType,
        currency: data.currency as Currency,
        fiatCurrency: data.fiatCurrency,
        pricePerUnit: data.pricePerUnit,
        minAmount: data.minAmount,
        maxAmount: data.maxAmount,
        totalAmount: data.totalAmount,
        paymentMethods: data.paymentMethods as PaymentMethodType[],
        paymentWindow: parseInt(data.paymentWindow),
        terms: data.terms,
      })
      await AsyncStorage.removeItem(DRAFT_KEY)
      trackEvent({ name: "offer_created", offerType: data.offerType, currency: data.currency })
      router.back()
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "validation") {
        setServerError(Object.values(apiErr.fields)[0] ?? "Validation error")
      } else {
        setServerError("Failed to create offer. Please try again.")
      }
    }
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1" edges={["bottom"]}>
        {/* Header */}
        <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
          <TouchableOpacity onPress={back} className="w-10 h-10 items-center justify-center -ml-2" activeOpacity={0.7}>
            <ChevronLeft size={24} color="#64748B" />
          </TouchableOpacity>
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Create offer
          </Text>
          <View className="w-10" />
        </View>

        {/* Step indicator */}
        <View className="px-5 pb-4">
          <View className="flex-row items-center justify-between">
            {STEPS.map((s, i) => {
              const isActive = step === s.num
              const isDone = step > s.num
              return (
                <View key={s.num} className="flex-row items-center flex-1">
                  <View
                    className={`w-8 h-8 rounded-full items-center justify-center ${
                      isDone ? "bg-brand" : isActive ? "bg-brand" : "bg-surface dark:bg-card-dark border border-border dark:border-border-dark"
                    }`}
                  >
                    {isDone ? (
                      <CheckCircle2 size={16} color="#FFFFFF" />
                    ) : (
                      <Text className={`text-xs font-bold ${isActive ? "text-white" : "text-muted dark:text-muted-dark"}`}>
                        {s.num}
                      </Text>
                    )}
                  </View>
                  {i < STEPS.length - 1 ? (
                    <View
                      className={`flex-1 h-0.5 mx-1.5 ${
                        step > s.num ? "bg-brand" : "bg-border dark:bg-border-dark"
                      }`}
                    />
                  ) : null}
                </View>
              )
            })}
          </View>
          <View className="flex-row mt-2 justify-between">
            {STEPS.map((s) => (
              <Text
                key={s.num}
                className={`text-[11px] flex-1 text-center ${
                  step >= s.num
                    ? "text-brand font-semibold"
                    : "text-muted dark:text-muted-dark"
                }`}
              >
                {s.label}
              </Text>
            ))}
          </View>
        </View>

        <ScrollView
          className="flex-1 px-5"
          keyboardShouldPersistTaps="handled"
          showsVerticalScrollIndicator={false}
        >
          {draftRestored ? (
            <View className="mb-4 rounded-lg bg-brand-bg px-4 py-2.5 flex-row items-center gap-2">
              <CheckCircle2 size={14} color="#00A3F6" />
              <Text className="text-xs text-brand">Draft restored.</Text>
            </View>
          ) : null}

          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          {/* ── STEP 1: ASSET ─────────────────────────────────────────── */}
          {step === 1 ? (
            <View>
              <SectionTitle title="What do you want to do?" subtitle="Pick an offer type and the asset you're trading." />

              <Label>I want to</Label>
              <View className="flex-row gap-3 mb-5">
                {(["buy", "sell"] as OfferType[]).map((t) => (
                  <TouchableOpacity
                    key={t}
                    onPress={() => setValue("offerType", t)}
                    activeOpacity={0.85}
                    className={`flex-1 rounded-xl py-4 items-center border ${
                      offerType === t
                        ? "bg-brand border-brand"
                        : "bg-surface dark:bg-card-dark border-border dark:border-border-dark"
                    }`}
                  >
                    <Text
                      className={`text-base font-semibold capitalize ${
                        offerType === t ? "text-white" : "text-foreground dark:text-foreground-dark"
                      }`}
                    >
                      {t === "buy" ? "Buy crypto" : "Sell crypto"}
                    </Text>
                    <Text
                      className={`text-xs mt-1 ${
                        offerType === t ? "text-white/80" : "text-muted dark:text-muted-dark"
                      }`}
                    >
                      {t === "buy" ? "Receive crypto for fiat" : "Sell crypto for fiat"}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>

              <Label>Cryptocurrency</Label>
              <View className="flex-row flex-wrap gap-2 mb-5">
                {CURRENCIES.map((c) => (
                  <TouchableOpacity
                    key={c}
                    onPress={() => setValue("currency", c)}
                    activeOpacity={0.85}
                    className={`px-4 py-2.5 rounded-xl border ${
                      currency === c
                        ? "bg-brand border-brand"
                        : "bg-surface dark:bg-card-dark border-border dark:border-border-dark"
                    }`}
                  >
                    <Text
                      className={`text-sm font-semibold ${
                        currency === c ? "text-white" : "text-foreground dark:text-foreground-dark"
                      }`}
                    >
                      {c}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>

              <Label>Fiat currency</Label>
              <View className="flex-row flex-wrap gap-2">
                {FIAT_OPTIONS.map((f) => (
                  <TouchableOpacity
                    key={f}
                    onPress={() => setValue("fiatCurrency", f)}
                    activeOpacity={0.85}
                    className={`px-4 py-2.5 rounded-xl border ${
                      fiatCurrency === f
                        ? "bg-brand border-brand"
                        : "bg-surface dark:bg-card-dark border-border dark:border-border-dark"
                    }`}
                  >
                    <Text
                      className={`text-sm font-semibold ${
                        fiatCurrency === f ? "text-white" : "text-foreground dark:text-foreground-dark"
                      }`}
                    >
                      {f}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>
              <FieldError msg={errors.fiatCurrency?.message} />
            </View>
          ) : null}

          {/* ── STEP 2: PRICE ─────────────────────────────────────────── */}
          {step === 2 ? (
            <View>
              <SectionTitle title="Set your price and limits" subtitle="Define how much you want to charge and trade sizes." />

              <Label>{`Price per ${currency}`}</Label>
              <Controller
                control={control}
                name="pricePerUnit"
                render={({ field: { onChange, onBlur, value } }) => (
                  <View className="relative mb-1">
                    <TextInput
                      className="h-14 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 pr-20 text-xl font-semibold text-foreground dark:text-foreground-dark"
                      onBlur={onBlur}
                      onChangeText={onChange}
                      value={String(value ?? "")}
                      placeholder="0.00"
                      placeholderTextColor="#94A3B8"
                      keyboardType="decimal-pad"
                      editable={!isSubmitting}
                    />
                    <View className="absolute right-4 top-4">
                      <Text className="text-sm font-semibold text-muted dark:text-muted-dark">
                        {fiatCurrency}
                      </Text>
                    </View>
                  </View>
                )}
              />
              <FieldError msg={errors.pricePerUnit?.message} />

              <View className="flex-row gap-3 mt-5">
                <View className="flex-1">
                  <Label>Min trade size</Label>
                  <Controller
                    control={control}
                    name="minAmount"
                    render={({ field: { onChange, onBlur, value } }) => (
                      <TextInput
                        className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-base text-foreground dark:text-foreground-dark"
                        onBlur={onBlur}
                        onChangeText={onChange}
                        value={String(value ?? "")}
                        placeholder="0.01"
                        placeholderTextColor="#94A3B8"
                        keyboardType="decimal-pad"
                        editable={!isSubmitting}
                      />
                    )}
                  />
                  <FieldError msg={errors.minAmount?.message} />
                </View>
                <View className="flex-1">
                  <Label>Max trade size</Label>
                  <Controller
                    control={control}
                    name="maxAmount"
                    render={({ field: { onChange, onBlur, value } }) => (
                      <TextInput
                        className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-base text-foreground dark:text-foreground-dark"
                        onBlur={onBlur}
                        onChangeText={onChange}
                        value={String(value ?? "")}
                        placeholder="1.0"
                        placeholderTextColor="#94A3B8"
                        keyboardType="decimal-pad"
                        editable={!isSubmitting}
                      />
                    )}
                  />
                  <FieldError msg={errors.maxAmount?.message} />
                </View>
              </View>

              <View className="mt-5">
                <Label>Total available</Label>
                <Controller
                  control={control}
                  name="totalAmount"
                  render={({ field: { onChange, onBlur, value } }) => (
                    <TextInput
                      className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-base text-foreground dark:text-foreground-dark"
                      onBlur={onBlur}
                      onChangeText={onChange}
                      value={String(value ?? "")}
                      placeholder="5.0"
                      placeholderTextColor="#94A3B8"
                      keyboardType="decimal-pad"
                      editable={!isSubmitting}
                    />
                  )}
                />
                <FieldError msg={errors.totalAmount?.message} />
                <Text className="text-xs text-muted dark:text-muted-dark mt-1.5">
                  Crypto locked for this offer until it's filled or paused.
                </Text>
              </View>
            </View>
          ) : null}

          {/* ── STEP 3: SETTINGS ──────────────────────────────────────── */}
          {step === 3 ? (
            <View>
              <SectionTitle title="Payment and terms" subtitle="How do you want to receive or send fiat?" />

              <Label>Payment methods</Label>
              <View className="flex-row flex-wrap gap-2 mb-5">
                {PAYMENT_METHODS.map(({ id, label }) => {
                  const selected = selectedPaymentMethods.includes(id)
                  return (
                    <TouchableOpacity
                      key={id}
                      onPress={() => togglePaymentMethod(id)}
                      activeOpacity={0.85}
                      className={`px-3.5 py-2.5 rounded-xl border ${
                        selected
                          ? "bg-brand border-brand"
                          : "bg-surface dark:bg-card-dark border-border dark:border-border-dark"
                      }`}
                    >
                      <Text
                        className={`text-sm font-semibold ${
                          selected ? "text-white" : "text-foreground dark:text-foreground-dark"
                        }`}
                      >
                        {label}
                      </Text>
                    </TouchableOpacity>
                  )
                })}
              </View>
              {errors.paymentMethods?.message ? (
                <Text className="-mt-3 mb-3 text-xs text-error">
                  {String(errors.paymentMethods.message)}
                </Text>
              ) : null}

              <Label>Payment window (minutes)</Label>
              <Controller
                control={control}
                name="paymentWindow"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={String(value ?? "")}
                    placeholder="30"
                    placeholderTextColor="#94A3B8"
                    keyboardType="number-pad"
                    editable={!isSubmitting}
                  />
                )}
              />
              <FieldError msg={errors.paymentWindow?.message} />
              <Text className="text-xs text-muted dark:text-muted-dark mt-1.5 mb-5">
                How long the buyer has to send payment before the trade expires.
              </Text>

              <Label>Trader's terms (optional)</Label>
              <Controller
                control={control}
                name="terms"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 py-3 text-sm text-foreground dark:text-foreground-dark h-28"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={String(value ?? "")}
                    placeholder="Describe your terms — bank details, hours, etc."
                    placeholderTextColor="#94A3B8"
                    multiline
                    textAlignVertical="top"
                    editable={!isSubmitting}
                  />
                )}
              />
            </View>
          ) : null}

          {/* ── STEP 4: REVIEW ────────────────────────────────────────── */}
          {step === 4 ? (
            <View>
              <SectionTitle title="Review and post" subtitle="Make sure everything looks right before publishing." />

              <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-3">
                <ReviewRow
                  label="Offer type"
                  value={formValues.offerType === "buy" ? "Buy crypto" : "Sell crypto"}
                />
                <ReviewRow label="Asset" value={formValues.currency ?? "—"} />
                <ReviewRow label="Fiat" value={formValues.fiatCurrency ?? "—"} />
                <ReviewRow
                  label="Price"
                  value={`${formValues.fiatCurrency} ${formValues.pricePerUnit ?? "—"}`}
                />
                <ReviewRow
                  label="Trade size"
                  value={`${formValues.minAmount ?? "—"} – ${formValues.maxAmount ?? "—"} ${formValues.currency ?? ""}`}
                />
                <ReviewRow
                  label="Total available"
                  value={`${formValues.totalAmount ?? "—"} ${formValues.currency ?? ""}`}
                />
                <ReviewRow
                  label="Payment methods"
                  value={
                    (formValues.paymentMethods ?? [])
                      .map((m) => PAYMENT_METHODS.find((p) => p.id === m)?.label ?? m)
                      .join(", ") || "—"
                  }
                />
                <ReviewRow
                  label="Payment window"
                  value={`${formValues.paymentWindow ?? "—"} min`}
                  last
                />
              </View>

              <View className="rounded-2xl bg-brand-bg p-4 flex-row items-center gap-2.5 mb-4">
                <ShieldCheck size={16} color="#00A3F6" />
                <Text className="text-xs text-brand flex-1">
                  Your crypto will be locked in escrow when a buyer starts a trade.
                </Text>
              </View>
            </View>
          ) : null}

          <View className="h-24" />
        </ScrollView>

        {/* Footer nav */}
        <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark flex-row gap-3 bg-background dark:bg-background-dark">
          {step > 1 ? (
            <TouchableOpacity
              onPress={back}
              className="px-5 h-12 rounded-xl border border-border dark:border-border-dark items-center justify-center flex-row gap-1.5"
              activeOpacity={0.85}
            >
              <ChevronLeft size={16} color="#64748B" />
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">Back</Text>
            </TouchableOpacity>
          ) : null}

          {step < 4 ? (
            <TouchableOpacity
              onPress={next}
              className="flex-1 h-12 rounded-xl bg-brand items-center justify-center flex-row gap-1.5"
              activeOpacity={0.85}
            >
              <Text className="text-base font-semibold text-white">Continue</Text>
              <ChevronRight size={16} color="#FFFFFF" />
            </TouchableOpacity>
          ) : (
            <TouchableOpacity
              onPress={handleSubmit(onSubmit)}
              disabled={isSubmitting}
              className="flex-1 h-12 rounded-xl bg-brand items-center justify-center"
              activeOpacity={0.85}
            >
              {isSubmitting ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text className="text-base font-semibold text-white">Post offer</Text>
              )}
            </TouchableOpacity>
          )}
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}

function SectionTitle({ title, subtitle }: { title: string; subtitle: string }) {
  return (
    <View className="mb-5">
      <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">{title}</Text>
      <Text className="text-sm text-muted dark:text-muted-dark mt-1">{subtitle}</Text>
    </View>
  )
}

function Label({ children }: { children: string }) {
  return (
    <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
      {children}
    </Text>
  )
}

function FieldError({ msg }: { msg?: string | undefined }) {
  if (!msg) return null
  return <Text className="mt-1 text-xs text-error">{msg}</Text>
}

function ReviewRow({ label, value, last = false }: { label: string; value: string; last?: boolean }) {
  return (
    <View
      className={`flex-row justify-between py-2.5 ${
        last ? "" : "border-b border-border/50 dark:border-border-dark/50"
      }`}
    >
      <Text className="text-sm text-muted dark:text-muted-dark">{label}</Text>
      <Text className="text-sm font-medium text-foreground dark:text-foreground-dark text-right flex-1 ml-3">
        {value}
      </Text>
    </View>
  )
}
