import {
  View, Text, TouchableOpacity, Image, ActivityIndicator, ScrollView, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import * as ImagePicker from "expo-image-picker"
import {
  ChevronLeft,
  ImagePlus,
  Camera,
  CheckCircle2,
  Receipt,
  X,
} from "lucide-react-native"
import { tradeService } from "@/src/services/trade.service"
import { useQueryClient } from "@tanstack/react-query"
import { stripExifAndCompress } from "@/src/lib/image-upload"

export default function ProofOfPaymentScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const router = useRouter()
  const qc = useQueryClient()
  const [image, setImage] = useState<{ uri: string; name: string } | null>(null)
  const [uploading, setUploading] = useState(false)
  const [uploaded, setUploaded] = useState(false)

  async function pickImage() {
    const { status } = await ImagePicker.requestMediaLibraryPermissionsAsync()
    if (status !== "granted") {
      Alert.alert("Permission required", "Please allow access to your photo library.")
      return
    }
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      quality: 1, // Strip + compress ourselves via expo-image-manipulator
    })
    if (!result.canceled && result.assets[0]) {
      const asset = result.assets[0]
      const stripped = await stripExifAndCompress(asset.uri)
      setImage({ uri: stripped.uri, name: stripped.name })
    }
  }

  async function takePhoto() {
    const { status } = await ImagePicker.requestCameraPermissionsAsync()
    if (status !== "granted") {
      Alert.alert("Permission required", "Please allow camera access.")
      return
    }
    const result = await ImagePicker.launchCameraAsync({
      allowsEditing: true,
      quality: 1,
    })
    if (!result.canceled && result.assets[0]) {
      const asset = result.assets[0]
      const stripped = await stripExifAndCompress(asset.uri)
      setImage({ uri: stripped.uri, name: stripped.name })
    }
  }

  async function handleUpload() {
    if (!image) return
    setUploading(true)
    try {
      const formData = new FormData()
      formData.append("file", {
        uri: image.uri,
        name: image.name,
        type: "image/jpeg",
      } as never)
      await tradeService.uploadProofOfPayment(id ?? "", formData)
      qc.invalidateQueries({ queryKey: ["trade", id] })
      setUploaded(true)
    } catch {
      Alert.alert("Upload failed", "Please try again.")
    } finally {
      setUploading(false)
    }
  }

  if (uploaded) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
        <View className="flex-1 items-center justify-center px-6">
          <View className="w-20 h-20 rounded-full bg-success/10 items-center justify-center mb-5">
            <CheckCircle2 size={36} color="#10B981" />
          </View>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            Proof uploaded
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8 max-w-[300px]">
            Your proof of payment has been sent to the seller. They'll release the escrow once they confirm receipt.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-xl bg-brand px-8 h-12 items-center justify-center"
            activeOpacity={0.85}
          >
            <Text className="text-base font-semibold text-white">Back to trade</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <View className="px-5 pt-2 pb-3 flex-row items-center">
        <TouchableOpacity
          onPress={() => router.back()}
          className="w-10 h-10 items-center justify-center -ml-2"
          activeOpacity={0.7}
        >
          <ChevronLeft size={24} color="#64748B" />
        </TouchableOpacity>
        <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
          Proof of payment
        </Text>
      </View>

      <ScrollView className="flex-1 px-5" showsVerticalScrollIndicator={false}>
        <View className="rounded-2xl bg-brand/10 border border-brand/20 p-4 mb-5 flex-row items-center gap-3">
          <View className="w-10 h-10 rounded-full bg-brand/20 items-center justify-center">
            <Receipt size={18} color="#00A3F6" />
          </View>
          <View className="flex-1">
            <Text className="text-sm font-semibold text-brand">Upload your receipt</Text>
            <Text className="text-xs text-brand/80 mt-0.5">
              A clear screenshot or photo helps the seller confirm faster.
            </Text>
          </View>
        </View>

        {image ? (
          <View className="mb-5">
            <View className="relative rounded-2xl overflow-hidden">
              <Image
                source={{ uri: image.uri }}
                className="w-full h-72"
                resizeMode="cover"
              />
              <TouchableOpacity
                onPress={() => setImage(null)}
                className="absolute top-3 right-3 w-9 h-9 rounded-full bg-black/60 items-center justify-center"
                activeOpacity={0.85}
              >
                <X size={16} color="#FFFFFF" />
              </TouchableOpacity>
            </View>
            <Text className="text-xs text-muted dark:text-muted-dark mt-2 text-center">
              EXIF data has been stripped for your privacy.
            </Text>
          </View>
        ) : (
          <View className="flex-row gap-3 mb-5">
            <TouchableOpacity
              onPress={pickImage}
              className="flex-1 rounded-2xl border-2 border-dashed border-brand/40 bg-brand/5 py-8 items-center"
              activeOpacity={0.7}
            >
              <View className="w-12 h-12 rounded-full bg-brand/15 items-center justify-center mb-2">
                <ImagePlus size={20} color="#00A3F6" />
              </View>
              <Text className="text-sm font-semibold text-brand">Photo library</Text>
            </TouchableOpacity>
            <TouchableOpacity
              onPress={takePhoto}
              className="flex-1 rounded-2xl border-2 border-dashed border-brand/40 bg-brand/5 py-8 items-center"
              activeOpacity={0.7}
            >
              <View className="w-12 h-12 rounded-full bg-brand/15 items-center justify-center mb-2">
                <Camera size={20} color="#00A3F6" />
              </View>
              <Text className="text-sm font-semibold text-brand">Camera</Text>
            </TouchableOpacity>
          </View>
        )}

        <View className="h-24" />
      </ScrollView>

      <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
        <TouchableOpacity
          onPress={handleUpload}
          disabled={!image || uploading}
          className={`rounded-xl h-12 items-center justify-center ${
            image ? "bg-brand" : "bg-brand/40"
          }`}
          activeOpacity={0.85}
        >
          {uploading ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text className="text-base font-semibold text-white">Upload proof</Text>
          )}
        </TouchableOpacity>
      </View>
    </SafeAreaView>
  )
}
