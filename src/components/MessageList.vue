<script setup>
defineProps({
  messages: Array
})

const emit = defineEmits(['delete-message'])

const deleteMessage = (id) => {
  emit('delete-message', id)
}
</script>

<template>
  <div class="messages-container">
    <transition-group name="list" tag="div" class="messages-list">
      <div 
        v-for="message in messages" 
        :key="message.id"
        class="message-card"
      >
        <div class="message-header">
          <span class="message-date">{{ message.date }}</span>
          <span class="message-time">{{ message.time }}</span>
        </div>
        
        <div class="message-content">
          {{ message.text }}
        </div>
        
        <div class="message-footer">
          <button 
            @click="deleteMessage(message.id)"
            class="btn-delete"
            title="删除此消息"
          >
            🗑️ 删除
          </button>
        </div>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.messages-container {
  width: 100%;
}

.messages-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;
  border-left: 4px solid #667eea;
}

.message-card:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  transform: translateY(-2px);
}

.message-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid #f0f0f0;
}

.message-date {
  font-weight: 600;
  color: #667eea;
}

.message-time {
  font-size: 0.85rem;
  color: #999;
}

.message-content {
  color: #333;
  line-height: 1.6;
  word-break: break-word;
  white-space: pre-wrap;
  margin-bottom: 1rem;
}

.message-footer {
  display: flex;
  justify-content: flex-end;
}

.btn-delete {
  padding: 0.4rem 0.8rem;
  background: rgba(255, 59, 48, 0.1);
  color: #ff3b30;
  border: 1px solid rgba(255, 59, 48, 0.3);
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.3s ease;
}

.btn-delete:hover {
  background: rgba(255, 59, 48, 0.2);
  border-color: rgba(255, 59, 48, 0.5);
}

/* 列表动画 */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-30px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
