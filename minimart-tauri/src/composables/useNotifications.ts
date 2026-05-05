import { reactive, readonly } from 'vue'

type ToastType = 'success' | 'error' | 'info' | 'warning'
type PromptType = 'danger' | 'info'

interface Toast {
  id: number
  type: ToastType
  title: string
  message?: string
}

interface PromptState {
  visible: boolean
  type: PromptType
  title: string
  message: string
  confirmText: string
  cancelText: string
  resolve?: (confirmed: boolean) => void
}

let nextToastId = 1

const state = reactive({
  toasts: [] as Toast[],
  prompt: {
    visible: false,
    type: 'info',
    title: '',
    message: '',
    confirmText: 'Confirm',
    cancelText: 'Cancel',
    resolve: undefined,
  } as PromptState,
})

const dismissToast = (id: number) => {
  const index = state.toasts.findIndex((toast) => toast.id === id)
  if (index >= 0) state.toasts.splice(index, 1)
}

const showToast = (title: string, message = '', type: ToastType = 'info', timeout = 4000) => {
  const id = nextToastId++
  state.toasts.push({ id, title, message, type })
  if (timeout > 0) window.setTimeout(() => dismissToast(id), timeout)
}

const showPrompt = (options: {
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: PromptType
}) => new Promise<boolean>((resolve) => {
  Object.assign(state.prompt, {
    visible: true,
    type: options.type || 'info',
    title: options.title,
    message: options.message,
    confirmText: options.confirmText || 'Confirm',
    cancelText: options.cancelText || 'Cancel',
    resolve,
  })
})

const closePrompt = (confirmed: boolean) => {
  state.prompt.resolve?.(confirmed)
  Object.assign(state.prompt, {
    visible: false,
    title: '',
    message: '',
    resolve: undefined,
  })
}

export const useNotifications = () => ({
  notifications: readonly(state),
  showToast,
  dismissToast,
  showPrompt,
  closePrompt,
})
