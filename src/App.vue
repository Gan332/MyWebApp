<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import MessageList from './components/MessageList.vue'
import MessageForm from './components/MessageForm.vue'

const messages = ref([])
const searchQuery = ref('')
const selectedTag = ref('全部')
const isDarkMode = ref(false)

// 初始化本地存储和深色模式
onMounted(() => {
  loadMessages()
  loadDarkMode()
})

// LocalStorage 功能
const loadMessages = () => {
  const saved = localStorage.getItem('messages')
  if (saved) {
    messages.value = JSON.parse(saved)
  }
}

const saveMessages = () => {
  localStorage.setItem('messages', JSON.stringify(messages.value))
}

const loadDarkMode = () => {
  const saved = localStorage.getItem('darkMode')
  if (saved) {
    isDarkMode.value = JSON.parse(saved)
  }
}

const toggleDarkMode = () => {
  isDarkMode.value = !isDarkMode.value
  localStorage.setItem('darkMode', JSON.stringify(isDarkMode.value))
}

// 监听消息变化，自动保存
watch(messages, saveMessages, { deep: true })

// 获取所有标签
const allTags = computed(() => {
  const tags = new Set(['全部'])
  messages.value.forEach(msg => {
    if (msg.tags && Array.isArray(msg.tags)) {
      msg.tags.forEach(tag => tags.add(tag))
    }
  })
  return Array.from(tags)
})

// 搜索和筛选
const filteredMessages = computed(() => {
  return messages.value.filter(msg => {
    const matchesSearch = msg.text.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesTag = selectedTag.value === '全部' || (msg.tags && msg.tags.includes(selectedTag.value))
    return matchesSearch && matchesTag
  })
})

const addMessage = (messageData) => {
  messages.value.unshift({
    id: Date.now(),
    text: messageData.text,
    tags: messageData.tags || [],
    date: new Date().toLocaleDateString('zh-CN'),
    time: new Date().toLocaleTimeString('zh-CN')
  })
}

const deleteMessage = (id) => {
  messages.value = messages.value.filter(msg => msg.id !== id)
}

const updateMessage = (id, updatedData) => {
  const msg = messages.value.find(m => m.id === id)
  if (msg) {
    Object.assign(msg, updatedData)
  }
}

const clearAll = () => {
  if (confirm('确定要清空所有消息吗？')) {
    messages.value = []
  }
}

// 导出为 JSON
const exportAsJSON = () => {
  const dataStr = JSON.stringify(messages.value, null, 2)
  const dataBlob = new Blob([dataStr], { type: 'application/json' })
  const url = URL.createObjectURL(dataBlob)
  const link = document.createElement('a')
  link.href = url
  link.download = `messages_${new Date().getTime()}.json`
  link.click()
  URL.revokeObjectURL(url)
}

// 导出为 CSV
const exportAsCSV = () => {
  const headers = ['ID', '消息内容', '标签', '日期', '时间']
  const rows = messages.value.map(msg => [
    msg.id,
    `"${msg.text.replace(/"/g, '""')}"`,
    (msg.tags || []).join(';'),
    msg.date,
    msg.time
  ])
  const csv = [headers, ...rows].map(row => row.join(',')).join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `messages_${new Date().getTime()}.csv`
  link.click()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <div :class="['app-container', { dark: isDarkMode }]">
    <header class="header">
      <div class="header-top">
        <h1>📱 每日数码消息收集</h1>
        <button @click="toggleDarkMode" class="btn-theme" :title="isDarkMode ? '切换亮色模式' : '切换深色模式'">
          {{ isDarkMode ? '☀️' : '🌙' }}
        </button>
      </div>
      <p class="subtitle">记录你每天的数码灵感和想法</p>
    </header>

    <main class="main-content">
      <MessageForm @add-message="addMessage" />
      
      <!-- 搜索和筛选 -->
      <div class="search-filter">
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="🔍 搜索消息..."
          class="search-input"
        />
        <div class="tag-filter">
          <button 
            v-for="tag in allTags" 
            :key="tag"
            @click="selectedTag = tag"
            :class="['tag-btn', { active: selectedTag === tag }]"
          >
            {{ tag }}
          </button>
        </div>
      </div>

      <!-- 统计和操作 -->
      <div class="stats-bar">
        <div class="stats">
          <span>📊 已收集 <strong>{{ messages.length }}</strong> 条消息</span>
          <span v-if="filteredMessages.length !== messages.length" class="filter-info">
            (筛选结果: <strong>{{ filteredMessages.length }}</strong> 条)
          </span>
        </div>
        <div class="export-buttons">
          <button @click="exportAsJSON" class="btn-export" title="导出为 JSON">📄 JSON</button>
          <button @click="exportAsCSV" class="btn-export" title="导出为 CSV">📊 CSV</button>
        </div>
      </div>

      <MessageList 
        :messages="filteredMessages" 
        @delete-message="deleteMessage"
        @update-message="updateMessage"
      />

      <div class="actions" v-if="messages.length > 0">
        <button @click="clearAll" class="btn-clear">🗑️ 清空所有消息</button>
      </div>

      <div class="empty-state" v-else>
        <p>暂无消息，开始添加你的第一条吧！</p>
      </div>
    </main>

    <footer class="footer">
      <p>© 2026 MyWebApp | 每日数码消息收集工具 | v2.0 增强版</p>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  transition: all 0.3s ease;
}

.app-container.dark {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  color: #e0e0e0;
}

.header {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  padding: 2rem;
  text-align: center;
  color: white;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.dark .header {
  background: rgba(0, 0, 0, 0.3);
}

.header-top {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 2rem;
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

.btn-theme {
  background: rgba(255, 255, 255, 0.2);
  border: 2px solid rgba(255, 255, 255, 0.3);
  color: white;
  padding: 0.8rem 1rem;
  border-radius: 8px;
  font-size: 1.2rem;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-theme:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.1);
}

.main-content {
  flex: 1;
  max-width: 800px;
  margin: 2rem auto;
  width: 100%;
  padding: 0 1rem;
}

/* 搜索和筛选 */
.search-filter {
  margin-bottom: 2rem;
  background: white;
  padding: 1.5rem;
  border-radius: 12px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
}

.dark .search-filter {
  background: #2a2a3e;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
}

.search-input {
  width: 100%;
  padding: 0.8rem;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 1rem;
  margin-bottom: 1rem;
  font-family: inherit;
  transition: border-color 0.3s;
}

.search-input:focus {
  outline: none;
  border-color: #667eea;
}

.dark .search-input {
  background: #333;
  color: #e0e0e0;
  border-color: #444;
}

.tag-filter {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.tag-btn {
  padding: 0.5rem 1rem;
  background: #f0f0f0;
  border: 2px solid #e0e0e0;
  border-radius: 20px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.3s ease;
}

.tag-btn:hover {
  background: #e0e0e0;
}

.tag-btn.active {
  background: #667eea;
  color: white;
  border-color: #667eea;
}

.dark .tag-btn {
  background: #444;
  border-color: #555;
  color: #e0e0e0;
}

.dark .tag-btn.active {
  background: #667eea;
  border-color: #667eea;
  color: white;
}

/* 统计和操作栏 */
.stats-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  flex-wrap: wrap;
  gap: 1rem;
}

.stats {
  color: white;
  font-size: 1.1rem;
}

.stats strong {
  font-size: 1.5rem;
  color: #ffd700;
}

.filter-info {
  margin-left: 1rem;
  opacity: 0.9;
}

.export-buttons {
  display: flex;
  gap: 0.5rem;
}

.btn-export {
  padding: 0.6rem 1rem;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 2px solid white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.3s ease;
}

.btn-export:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-2px);
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

.dark .footer {
  background: rgba(0, 0, 0, 0.5);
}

.footer p {
  margin: 0;
  opacity: 0.8;
}

@media (max-width: 768px) {
  .header h1 {
    font-size: 2rem;
  }

  .header-top {
    gap: 1rem;
  }

  .stats-bar {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
