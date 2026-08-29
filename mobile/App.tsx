/**
 * QicTrader Android app — native shell around the QicTrader web app.
 *
 * Architecture decision (2026-07-05): the product changes too fast for a
 * parallel native rewrite to stay 1:1 with web. This shell renders the
 * real web frontend (staging by default, see src/config.ts) inside a
 * WebView and adds the native affordances a wrapper must provide:
 *
 *  - session persistence (cookies + localStorage survive restarts)
 *  - Android hardware back = web history back, double-back to exit
 *  - camera / mic runtime permissions for KYC liveness (getUserMedia)
 *  - <input type="file"> uploads (payment proof, KYC docs) incl. camera
 *  - external links (WhatsApp, banks, explorers, mailto:, tel:) open in
 *    the right native app instead of inside the shell
 *  - pull-to-refresh, offline/error screen with retry
 *  - branded splash held until the web app has actually painted
 *
 * Web deploys are instantly reflected here — no app-store release needed
 * for UI/feature changes.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import {
  ActivityIndicator,
  BackHandler,
  Linking,
  PermissionsAndroid,
  Platform,
  StyleSheet,
  Text,
  ToastAndroid,
  TouchableOpacity,
  View,
} from "react-native";
import {
  SafeAreaProvider,
  SafeAreaView,
} from "react-native-safe-area-context";
import { StatusBar } from "expo-status-bar";
import * as SplashScreen from "expo-splash-screen";
import { WebView } from "react-native-webview";
import type {
  ShouldStartLoadRequest,
  WebViewErrorEvent,
  WebViewNavigation,
} from "react-native-webview/lib/WebViewTypes";

import {
  INTERNAL_HOSTS,
  KYC_PATH_HINTS,
  USER_AGENT_SUFFIX,
  WEB_APP_URL,
} from "./src/config";

// Keep the native splash up until the web app has painted its first page.
SplashScreen.preventAutoHideAsync().catch(() => {
  /* already hidden — nothing to do */
});

const BRAND_BLUE = "#00A3F6";

function hostOf(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return "";
  }
}

function isInternal(url: string): boolean {
  if (!/^https?:\/\//i.test(url)) return false;
  const host = hostOf(url);
  return INTERNAL_HOSTS.some((re) => re.test(host));
}

export default function App() {
  const webRef = useRef<WebView>(null);
  const canGoBackRef = useRef(false);
  const lastBackPressRef = useRef(0);
  const kycPermsRequestedRef = useRef(false);
  const [loadFailed, setLoadFailed] = useState<string | null>(null);
  const [webKey, setWebKey] = useState(0);

  // ── Android hardware back: mirror browser history ──────────────────
  useEffect(() => {
    const sub = BackHandler.addEventListener("hardwareBackPress", () => {
      if (canGoBackRef.current) {
        webRef.current?.goBack();
        return true;
      }
      const now = Date.now();
      if (now - lastBackPressRef.current < 2000) {
        return false; // second press within 2s → let Android exit
      }
      lastBackPressRef.current = now;
      if (Platform.OS === "android") {
        ToastAndroid.show("Press back again to exit QicTrader", ToastAndroid.SHORT);
      }
      return true;
    });
    return () => sub.remove();
  }, []);

  // ── Camera/mic runtime permissions ahead of KYC liveness ───────────
  const ensureKycPermissions = useCallback(async () => {
    if (Platform.OS !== "android" || kycPermsRequestedRef.current) return;
    kycPermsRequestedRef.current = true;
    try {
      await PermissionsAndroid.requestMultiple([
        PermissionsAndroid.PERMISSIONS.CAMERA,
        PermissionsAndroid.PERMISSIONS.RECORD_AUDIO,
      ]);
    } catch (e) {
      // Permission dialog failure is non-fatal: the web flow will surface
      // its own "camera unavailable" message and the user can retry from
      // Android Settings. Log so it shows up in adb logcat.
      console.warn("KYC permission request failed", e);
    }
  }, []);

  const onNavigationStateChange = useCallback(
    (nav: WebViewNavigation) => {
      canGoBackRef.current = nav.canGoBack;
      if (nav.url && KYC_PATH_HINTS.test(nav.url)) {
        void ensureKycPermissions();
      }
    },
    [ensureKycPermissions]
  );

  // ── Keep qictrader in-app; hand everything else to the OS ──────────
  const onShouldStartLoadWithRequest = useCallback(
    (request: ShouldStartLoadRequest): boolean => {
      const { url } = request;
      if (isInternal(url)) return true;
      // Non-http schemes (mailto:, tel:, whatsapp:, intent:, upi:, …)
      // and third-party http(s) links open outside the shell.
      Linking.openURL(url).catch((e) =>
        console.warn(`Could not open external URL ${url}`, e)
      );
      return false;
    },
    []
  );

  const onError = useCallback((event: WebViewErrorEvent) => {
    setLoadFailed(event.nativeEvent.description || "Connection failed");
    SplashScreen.hideAsync().catch(() => {});
  }, []);

  const onLoadEnd = useCallback(() => {
    SplashScreen.hideAsync().catch(() => {});
  }, []);

  const retry = useCallback(() => {
    setLoadFailed(null);
    setWebKey((k) => k + 1); // remount the WebView for a clean reload
  }, []);

  return (
    <SafeAreaProvider>
      <SafeAreaView style={styles.safeArea} edges={["top", "bottom"]}>
        <StatusBar style="dark" />
        {loadFailed ? (
          <View style={styles.errorContainer}>
            <Text style={styles.errorTitle}>Can't reach QicTrader</Text>
            <Text style={styles.errorDetail}>
              {loadFailed}. Check your connection and try again.
            </Text>
            <TouchableOpacity style={styles.retryButton} onPress={retry}>
              <Text style={styles.retryLabel}>Retry</Text>
            </TouchableOpacity>
          </View>
        ) : (
          <WebView
            key={webKey}
            ref={webRef}
            source={{ uri: WEB_APP_URL }}
            style={styles.webview}
            // ── 1:1 rendering ────────────────────────────────────────
            textZoom={100} // ignore Android font scaling so layout matches web exactly
            setSupportMultipleWindows={false} // target=_blank routes through onShouldStartLoadWithRequest
            // ── session persistence ─────────────────────────────────
            domStorageEnabled
            sharedCookiesEnabled
            thirdPartyCookiesEnabled
            // ── media / KYC ──────────────────────────────────────────
            mediaPlaybackRequiresUserAction={false}
            allowsInlineMediaPlayback
            // ── navigation ───────────────────────────────────────────
            onShouldStartLoadWithRequest={onShouldStartLoadWithRequest}
            onNavigationStateChange={onNavigationStateChange}
            allowsBackForwardNavigationGestures
            pullToRefreshEnabled
            // ── lifecycle ────────────────────────────────────────────
            onLoadEnd={onLoadEnd}
            onError={onError}
            onRenderProcessGone={retry} // Android WebView process died → clean remount
            startInLoadingState
            renderLoading={() => (
              <View style={styles.loading}>
                <ActivityIndicator size="large" color={BRAND_BLUE} />
              </View>
            )}
            applicationNameForUserAgent={USER_AGENT_SUFFIX}
          />
        )}
      </SafeAreaView>
    </SafeAreaProvider>
  );
}

const styles = StyleSheet.create({
  safeArea: {
    flex: 1,
    backgroundColor: "#FFFFFF",
  },
  webview: {
    flex: 1,
    backgroundColor: "#FFFFFF",
  },
  loading: {
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#FFFFFF",
  },
  errorContainer: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    paddingHorizontal: 32,
    backgroundColor: "#FFFFFF",
  },
  errorTitle: {
    fontSize: 20,
    fontWeight: "600",
    color: "#111111",
    marginBottom: 8,
  },
  errorDetail: {
    fontSize: 14,
    color: "#666666",
    textAlign: "center",
    marginBottom: 24,
  },
  retryButton: {
    backgroundColor: BRAND_BLUE,
    borderRadius: 12,
    paddingHorizontal: 32,
    paddingVertical: 12,
  },
  retryLabel: {
    color: "#FFFFFF",
    fontSize: 16,
    fontWeight: "600",
  },
});
