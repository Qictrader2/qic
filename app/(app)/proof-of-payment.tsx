import {
  View, Text, TouchableOpacity, Image, ActivityIndicator, ScrollView, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import * as ImagePicker from "expo-image-picker"
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
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
        <View className="items-center">
          <Text className="text-5xl mb-4">✅</Text>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            Proof uploaded
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8">
            Your proof of payment has been sent to the seller.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-lg bg-brand px-8 py-3.5"
          >
            <Text className="text-base font-semibold text-white">Back to trade</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
          Proof of Payment
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-6 leading-relaxed">
          Upload a screenshot or photo showing your payment. This helps the seller confirm
          they have received funds.
        </Text>

        {image ? (
          <View className="mb-4">
            <Image
              source={{ uri: image.uri }}
              className="w-full h-56 rounded-xl"
              resizeMode="cover"
            />
            <TouchableOpacity
              onPress={() => setImage(null)}
              className="mt-2 items-center py-2"
            >
              <Text className="text-sm text-error">Remove</Text>
            </TouchableOpacity>
          </View>
        ) : (
          <View className="flex-row gap-3 mb-6">
            <TouchableOpacity
              onPress={pickImage}
              className="flex-1 rounded-xl border-2 border-dashed border-brand/50 bg-brand-bg/30 py-8 items-center"
              activeOpacity={0.7}
            >
              <Text className="text-2xl mb-2">🖼</Text>
              <Text className="text-sm font-medium text-brand">Photo library</Text>
            </TouchableOpacity>
            <TouchableOpacity
              onPress={takePhoto}
              className="flex-1 rounded-xl border-2 border-dashed border-brand/50 bg-brand-bg/30 py-8 items-center"
              activeOpacity={0.7}
            >
              <Text className="text-2xl mb-2">📷</Text>
              <Text className="text-sm font-medium text-brand">Camera</Text>
            </TouchableOpacity>
          </View>
        )}

        <TouchableOpacity
          onPress={handleUpload}
          disabled={!image || uploading}
          className={`rounded-lg py-4 items-center ${image ? "bg-brand" : "bg-muted/30"}`}
          activeOpacity={0.8}
        >
          {uploading ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text className={`text-base font-semibold ${image ? "text-white" : "text-muted"}`}>
              Upload proof
            </Text>
          )}
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  )
}
