// Re-export all types from separate files
export * from "./auth.types"
export * from "./project.types"
export * from "./ui.types"
export * from "./api.types"
export * from "./user.types"
export * from "./trade.types"
export * from "./wallet.types"
export * from "./escrow.types"
export * from "./notification.types"
export * from "./affiliate.types"
export * from "./payment-method.types"
export * from "./reseller.types"
export * from "./offer.types"
export * from "./kyc.types"
export * from "./whatsapp.types"
export * from "./two-factor.types"
export * from "./direct-trade.types"

// Export Firebase types (excluding UserSettings which is defined in user.types.ts)
export type {
  FirestoreDocument,
  FirebaseTimestamp,
  FirestoreUser,
  Product,
  Post,
  Message,
  ChatRoom,
  // UserSettings - using the one from user.types.ts instead
  SiteStats,
  Task,
  Comment,
  ActivityLog,
  PaginatedResult,
  OrderDirection,
  WhereOperator,
  FirebaseError,
  FirebaseResponse,
  BatchCreateOperation,
  BatchUpdateOperation,
  BatchDeleteOperation,
  BatchOperation,
  UseFirestoreResult,
  UseFirestoreListResult,
  UserPresence,
  FileMetadata,
  StorageFileInfo,
  StorageUploadOptions,
  StorageUploadProgress,
  StorageFolder,
  ValidationResult,
  UseStorageUploadResult,
  UploadOptions,
  FileInfo,
  UploadProgress,
  RealtimeQueryOptions,
  RealtimeSubscriptionOptions,
  QueryOptions,
  PaginationOptions,
} from "./firebase.types"
