import { View, Animated } from "react-native"
import { useEffect, useRef } from "react"

function Pulse({ className }: { className?: string }) {
  const opacity = useRef(new Animated.Value(0.4)).current

  useEffect(() => {
    const anim = Animated.loop(
      Animated.sequence([
        Animated.timing(opacity, { toValue: 0.8, duration: 700, useNativeDriver: true }),
        Animated.timing(opacity, { toValue: 0.4, duration: 700, useNativeDriver: true }),
      ])
    )
    anim.start()
    return () => anim.stop()
  }, [])

  return (
    <Animated.View
      style={{ opacity }}
      className={`bg-muted/20 dark:bg-muted-dark/20 rounded-md ${className ?? ""}`}
    />
  )
}

export function WalletCardSkeleton() {
  return (
    <View className="rounded-xl bg-surface dark:bg-surface-dark p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between mb-3">
        <View className="flex-row items-center gap-2">
          <Pulse className="h-10 w-10 rounded-full" />
          <View>
            <Pulse className="h-3.5 w-16 mb-1.5" />
            <Pulse className="h-3 w-12" />
          </View>
        </View>
        <View className="items-end">
          <Pulse className="h-4 w-24 mb-1" />
          <Pulse className="h-3 w-16" />
        </View>
      </View>
      <View className="flex-row gap-2">
        <Pulse className="flex-1 h-9 rounded-lg" />
        <Pulse className="flex-1 h-9 rounded-lg" />
      </View>
    </View>
  )
}

export function OfferRowSkeleton() {
  return (
    <View className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between mb-2">
        <View className="flex-row items-center gap-2">
          <Pulse className="h-5 w-10 rounded-full" />
          <Pulse className="h-4 w-12" />
        </View>
        <Pulse className="h-5 w-24" />
      </View>
      <View className="flex-row items-center justify-between">
        <Pulse className="h-3 w-32" />
        <Pulse className="h-3 w-28" />
      </View>
    </View>
  )
}

export function TradeRowSkeleton() {
  return (
    <View className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between mb-2">
        <View className="flex-row items-center gap-2">
          <Pulse className="h-5 w-20 rounded-full" />
          <Pulse className="h-3.5 w-12" />
        </View>
        <Pulse className="h-4 w-20" />
      </View>
      <View className="flex-row justify-between">
        <Pulse className="h-3 w-24" />
        <Pulse className="h-3 w-20" />
      </View>
    </View>
  )
}

export function ListSkeleton({
  count = 5,
  Item,
}: {
  count?: number
  Item: React.ComponentType
}) {
  return (
    <>
      {Array.from({ length: count }).map((_, i) => (
        <Item key={i} />
      ))}
    </>
  )
}
