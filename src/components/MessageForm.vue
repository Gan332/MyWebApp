<script setup>
import { ref } from 'vue'

const messageInput = ref('')

const emit = defineEmits(['add-message'])

const handleSubmit = () => {
  if (messageInput.value.trim()) {
    emit('add-message', messageInput.value.trim())
    messageInput.value = ''
  }
}

const handleKeydown = (e) => {
  if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
    handleSubmit()
  }
}
</script>

<template>
  <div class="form-container">
    <div class="form-group">
      <textarea
        v-model="messageInput"
        placeholder="📝 输入你的数码消息... (Ctrl+Enter 或 Cmd+Enter 快速提交)"
        @keydown="handleKeydown"
        class="message-input"
      ></textarea>
      <button 
        @click="handleSubmit"
        class="btn-submit"
        :disabled="!messageInput.trim()"
      >
        ✨ 添加消息
      </button>
    </div>
  </div>
</template>

<style scoped>
.form-container {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message-input {
  width: 100%;
  padding: 1rem;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 1rem;
  font-family: inherit;
  resize: vertical;
  min-height: 100px;
  transition: border-color 0.3s ease;
}

.message-input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.btn-submit {
  padding: 0.8rem 1.5rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-submit:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(102, 126, 234, 0.4);
}

.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-submit:active:not(:disabled) {
  transform: translateY(0);
}
</style>
