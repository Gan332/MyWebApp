<script setup>
import { ref } from 'vue'
import MessageList from './components/MessageList.vue'
import MessageForm from './components/MessageForm.vue'

const messages = ref([])

const addMessage = (messageText) => {
  messages.value.unshift({
    id: Date.now(),
    text: messageText,
    date: new Date().toLocaleDateString('zh-CN'),
    time: new Date().toLocaleTimeString('zh-CN')
  })
}

const deleteMessage = (id) => {
  messages.value = messages.value.filter(msg => msg.id !== id)
}

const clearAll = () => {
  if (confirm('确定要清空所有消息吗？')) {
    messages.value = []
  }
}
</script>

<template>
  <div class="app-container">
    <header class="header">
      <h1>📱 每日数码消息收集</h1>
      <p class="subtitle">记录你每天的数码灵感和想法</p>
    </header>

    <main class="main-content">
      <MessageForm @add-message="addMessage" />
      
      <div class="stats">
        <span>📊 已收集 <strong>{{ messages.length }}</strong> 条消息</span>
      </div>

      <MessageList 
        :messages="messages" 
        @delete-message="deleteMessage"
      />

      <div class="actions" v-if="messages.length > 0">
        <button @click="clearAll" class="btn-clear">🗑️ 清空所有消息</button>
      </div>

      <div class="empty-state" v-else>
        <p>暂无消息，开始添加你的第一条吧！</p>
      </div>
    </main>

    <footer class="footer">
      <p>© 2026 MyWebApp | 每日数码消息收集工具</p>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.header {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  padding: 2rem;
  text-align: center;
  color: white;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.header h1 {
  margin: 0;
  font-size: 2.5rem;
  font-weight: 700;
}

.subtitle {
  margin: 0.5rem 0 0;
  font-size: 1rem;
  opacity: 0.9;
}

.main-content {
  flex: 1;
  max-width: 800px;
  margin: 2rem auto;
  width: 100%;
  padding: 0 1rem;
}

.stats {
  text-align: center;
  color: white;
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

.stats strong {
  font-size: 1.5rem;
  color: #ffd700;
}

.actions {
  text-align: center;
  margin-top: 2rem;
}

.btn-clear {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 2px solid white;
  padding: 0.8rem 1.5rem;
  border-radius: 8px;
  cursor: pointer;
  font-size: 1rem;
  transition: all 0.3s ease;
}

.btn-clear:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.05);
}

.empty-state {
  text-align: center;
  padding: 3rem;
  color: rgba(255, 255, 255, 0.7);
  font-size: 1.1rem;
}

.footer {
  background: rgba(0, 0, 0, 0.2);
  color: white;
  text-align: center;
  padding: 1.5rem;
  margin-top: 2rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.footer p {
  margin: 0;
  opacity: 0.8;
}
</style>
