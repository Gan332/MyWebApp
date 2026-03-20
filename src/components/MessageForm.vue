<script setup>
import { ref } from 'vue'

const messageInput = ref('')
const tagInput = ref('')
const tags = ref([])

const emit = defineEmits(['add-message'])

const addTag = () => {
  if (tagInput.value.trim() && !tags.value.includes(tagInput.value.trim())) {
    tags.value.push(tagInput.value.trim())
    tagInput.value = ''
  }
}

const removeTag = (index) => {
  tags.value.splice(index, 1)
}

const handleSubmit = () => {
  if (messageInput.value.trim()) {
    emit('add-message', {
      text: messageInput.value.trim(),
      tags: tags.value
    })
    messageInput.value = ''
    tags.value = []
  }
}

const handleKeydown = (e) => {
  if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
    handleSubmit()
  }
}

const handleTagKeydown = (e) => {
  if (e.key === 'Enter') {
    e.preventDefault()
    addTag()
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
      
      <!-- 标签系统 -->
      <div class="tag-system">
        <div class="tag-input-group">
          <input 
            v-model="tagInput"
            type="text"
            placeholder="🏷️ 添加标签... (按 Enter 增加)"
            @keydown="handleTagKeydown"
            class="tag-input"
          />
          <button @click="addTag" class="btn-add-tag">+</button>
        </div>
        <div class="tags-display">
          <span 
            v-for="(tag, index) in tags"
            :key="index"
            class="tag-badge"
          >
            {{ tag }}
            <button @click="removeTag(index)" class="btn-remove-tag">×</button>
          </span>
        </div>
      </div>

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

/* 标签系统 */
.tag-system {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tag-input-group {
  display: flex;
  gap: 0.5rem;
}

.tag-input {
  flex: 1;
  padding: 0.6rem;
  border: 2px solid #e0e0e0;
  border-radius: 6px;
  font-size: 0.9rem;
  transition: border-color 0.3s;
}

.tag-input:focus {
  outline: none;
  border-color: #667eea;
}

.btn-add-tag {
  padding: 0.6rem 1rem;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: bold;
  transition: all 0.3s ease;
}

.btn-add-tag:hover {
  background: #764ba2;
  transform: scale(1.05);
}

.tags-display {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.tag-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  background: #f0f0f0;
  color: #333;
  padding: 0.4rem 0.8rem;
  border-radius: 20px;
  font-size: 0.9rem;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-10px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.btn-remove-tag {
  background: none;
  border: none;
  color: #999;
  cursor: pointer;
  font-size: 1.1rem;
  padding: 0;
  transition: color 0.2s;
}

.btn-remove-tag:hover {
  color: #ff6b6b;
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
