import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import type { AuthUser } from "@/src/store/auth-store"

interface AuthSliceState {
  user: AuthUser | null
  isAuthenticated: boolean
  isLoading: boolean
}

const initialState: AuthSliceState = {
  user: null,
  isAuthenticated: false,
  isLoading: true,
}

const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {
    setUser(state, action: PayloadAction<AuthUser | null>) {
      state.user = action.payload
      state.isAuthenticated = !!action.payload
    },
    setLoading(state, action: PayloadAction<boolean>) {
      state.isLoading = action.payload
    },
    clearAuth(state) {
      state.user = null
      state.isAuthenticated = false
      state.isLoading = false
    },
  },
})

export const { setUser, setLoading, clearAuth } = authSlice.actions
export default authSlice.reducer
