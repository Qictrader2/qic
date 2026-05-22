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
import { useState } from "react"
import { useCreateOffer } from "@/src/hooks/api/use-market"
import type { OfferType, Currency, PaymentMethodType } from "@/src/services/market.service"
import { ApiError } from "@/src/lib/api/client"

const CURRENCIES: Currency[] = ["BTC", "ETH", "SOL", "USDT", "USDC"]
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

export default function CreateOfferScreen() {
  const router = useRouter()
  const { mutateAsync: createOffer } = useCreateOffer()
  const [serverError, setServerError] = useState<string | null>(null)

  const {
    control,
    handleSubmit,
    watch,
    setValue,
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
  })

  const offerType = watch("offerType")
  const selectedCurrency = watch("currency")
  const selectedPaymentMethods = watch("paymentMethods") ?? []

  function togglePaymentMethod(id: string) {
    const current = selectedPaymentMethods
    if (current.includes(id)) {
      setValue("paymentMethods", current.filter((m) => m !== id))
    } else {
      setValue("paymentMethods", [...current, id])
    }
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

  function FieldError({ name }: { name: keyof typeof errors }) {
    const msg = errors[name]?.message
    return msg ? <Text className="mt-1 text-xs text-error">{String(msg)}</Text> : null
  }

  function Label({ children }: { children: string }) {
    return (
      <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
        {children}
      </Text>
    )
  }

  function Input({
    name,
    placeholder,
    keyboardType = "default",
    multiline = false,
  }: {
    name: keyof Form
    placeholder: string
    keyboardType?: "default" | "decimal-pad" | "number-pad"
    multiline?: boolean
  }) {
    return (
      <Controller
        control={control}
        name={name}
        render={({ field: { onChange, onBlur, value } }) => (
          <TextInput
            className={`rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark ${multiline ? "h-24" : ""}`}
            onBlur={onBlur}
            onChangeText={onChange}
            value={String(value ?? "")}
            placeholder={placeholder}
            placeholderTextColor="#94A3B8"
            keyboardType={keyboardType}
            multiline={multiline}
            textAlignVertical={multiline ? "top" : "center"}
            editable={!isSubmitting}
          />
        )}
      />
    )
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1">
        <ScrollView className="flex-1 px-4 py-4" keyboardShouldPersistTaps="handled">
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-6">
            Create Offer
          </Text>

          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          {/* Buy / Sell */}
          <View className="mb-4">
            <Label>I want to</Label>
            <View className="flex-row gap-3">
              {(["buy", "sell"] as OfferType[]).map((t) => (
                <TouchableOpacity
                  key={t}
                  onPress={() => setValue("offerType", t)}
                  className={`flex-1 rounded-lg py-3 items-center border ${
                    offerType === t
                      ? t === "buy"
                        ? "bg-success-bg border-success"
                        : "bg-error-bg border-error"
                      : "border-border dark:border-border-dark"
                  }`}
                >
                  <Text
                    className={`text-sm font-semibold capitalize ${
                      offerType === t
                        ? t === "buy"
                          ? "text-success"
                          : "text-error"
                        : "text-muted dark:text-muted-dark"
                    }`}
                  >
                    {t}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Currency */}
          <View className="mb-4">
            <Label>Cryptocurrency</Label>
            <View className="flex-row flex-wrap gap-2">
              {CURRENCIES.map((c) => (
                <TouchableOpacity
                  key={c}
                  onPress={() => setValue("currency", c)}
                  className={`px-4 py-2 rounded-lg border ${
                    selectedCurrency === c
                      ? "bg-brand border-brand"
                      : "border-border dark:border-border-dark"
                  }`}
                >
                  <Text className={`text-sm font-medium ${selectedCurrency === c ? "text-white" : "text-foreground dark:text-foreground-dark"}`}>
                    {c}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Fiat */}
          <View className="mb-4">
            <Label>Fiat currency (e.g. ZAR, USD)</Label>
            <Input name="fiatCurrency" placeholder="ZAR" />
            <FieldError name="fiatCurrency" />
          </View>

          {/* Price */}
          <View className="mb-4">
            <Label>Price per unit</Label>
            <Input name="pricePerUnit" placeholder="1000.00" keyboardType="decimal-pad" />
            <FieldError name="pricePerUnit" />
          </View>

          {/* Limits */}
          <View className="flex-row gap-3 mb-4">
            <View className="flex-1">
              <Label>Min amount</Label>
              <Input name="minAmount" placeholder="0.01" keyboardType="decimal-pad" />
              <FieldError name="minAmount" />
            </View>
            <View className="flex-1">
              <Label>Max amount</Label>
              <Input name="maxAmount" placeholder="1.0" keyboardType="decimal-pad" />
              <FieldError name="maxAmount" />
            </View>
          </View>

          <View className="mb-4">
            <Label>Total available</Label>
            <Input name="totalAmount" placeholder="5.0" keyboardType="decimal-pad" />
            <FieldError name="totalAmount" />
          </View>

          {/* Payment methods */}
          <View className="mb-4">
            <Label>Payment methods</Label>
            <View className="flex-row flex-wrap gap-2">
              {PAYMENT_METHODS.map(({ id, label }) => {
                const selected = selectedPaymentMethods.includes(id)
                return (
                  <TouchableOpacity
                    key={id}
                    onPress={() => togglePaymentMethod(id)}
                    className={`px-3 py-2 rounded-lg border ${
                      selected ? "bg-brand-bg border-brand" : "border-border dark:border-border-dark"
                    }`}
                  >
                    <Text className={`text-xs font-medium ${selected ? "text-brand" : "text-muted dark:text-muted-dark"}`}>
                      {label}
                    </Text>
                  </TouchableOpacity>
                )
              })}
            </View>
            {errors.paymentMethods?.message ? (
              <Text className="mt-1 text-xs text-error">{String(errors.paymentMethods.message)}</Text>
            ) : null}
          </View>

          {/* Payment window */}
          <View className="mb-4">
            <Label>Payment window (minutes)</Label>
            <Input name="paymentWindow" placeholder="30" keyboardType="number-pad" />
            <FieldError name="paymentWindow" />
          </View>

          {/* Terms */}
          <View className="mb-6">
            <Label>Terms (optional)</Label>
            <Input name="terms" placeholder="Describe your terms…" multiline />
          </View>

          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="rounded-lg bg-brand py-4 items-center mb-8"
            activeOpacity={0.8}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Post offer</Text>
            )}
          </TouchableOpacity>
        </ScrollView>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
