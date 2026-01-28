import { useState, useRef, useEffect } from 'react'
import { Send, Smile, Clock, Brain, ThumbsUp, ThumbsDown } from 'lucide-react'

interface Message {
  id: number
  content: string
  sender: 'user' | 'system'
  timestamp: Date
}

const Conversation = () => {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: 1,
      content: '你好张先生！今天感觉怎么样？',
      sender: 'system',
      timestamp: new Date(Date.now() - 3600000),
    },
    {
      id: 2,
      content: '还不错，早上去公园散步了。',
      sender: 'user',
      timestamp: new Date(Date.now() - 3500000),
    },
    {
      id: 3,
      content: '那太好了！散步对身心健康都有好处。还记得昨天做了什么吗？',
      sender: 'system',
      timestamp: new Date(Date.now() - 3400000),
    },
  ])

  const [newMessage, setNewMessage] = useState('')
  const [conversationStats] = useState({
    duration: 15,
    moodScore: 7,
    engagementScore: 8,
    cognitiveScore: 78,
  })

  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSendMessage = () => {
    if (!newMessage.trim()) return

    const userMessage: Message = {
      id: messages.length + 1,
      content: newMessage.trim(),
      sender: 'user',
      timestamp: new Date(),
    }

    setMessages([...messages, userMessage])
    setNewMessage('')

    // 模拟系统回复
    setTimeout(() => {
      const responses = [
        '我明白了。还有其他想分享的吗？',
        '听起来很有趣！能多告诉我一些吗？',
        '谢谢分享。今天天气怎么样？',
        '你还记得小时候最喜欢做什么吗？',
        '如果明天要出门，你需要准备什么？',
      ]

      const systemMessage: Message = {
        id: messages.length + 2,
        content: responses[Math.floor(Math.random() * responses.length)],
        sender: 'system',
        timestamp: new Date(),
      }

      setMessages(prev => [...prev, systemMessage])
    }, 1000)
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }

  const handleEndConversation = () => {
    // 这里应该调用API结束对话并保存分析结果
    alert('对话已结束，分析结果已保存')
  }

  return (
    <div className="space-y-6">
      {/* 标题和统计 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">日常对话</h1>
          <p className="text-gray-600 mt-2">通过自然对话进行认知健康监测</p>
        </div>

        <div className="flex items-center space-x-4">
          <div className="text-center">
            <div className="flex items-center text-sm text-gray-500">
              <Clock className="w-4 h-4 mr-1" />
              时长
            </div>
            <div className="text-lg font-semibold">{conversationStats.duration}分钟</div>
          </div>

          <div className="text-center">
            <div className="flex items-center text-sm text-gray-500">
              <Smile className="w-4 h-4 mr-1" />
              情绪
            </div>
            <div className="text-lg font-semibold">{conversationStats.moodScore}/10</div>
          </div>

          <div className="text-center">
            <div className="flex items-center text-sm text-gray-500">
              <Brain className="w-4 h-4 mr-1" />
              认知分数
            </div>
            <div className="text-lg font-semibold">{conversationStats.cognitiveScore}</div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 对话区域 */}
        <div className="lg:col-span-2">
          <div className="card h-[600px] flex flex-col">
            {/* 消息列表 */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {messages.map((message) => (
                <div
                  key={message.id}
                  className={`flex ${message.sender === 'user' ? 'justify-end' : 'justify-start'}`}
                >
                  <div
                    className={`max-w-[70%] rounded-2xl px-4 py-3 ${message.sender === 'user'
                        ? 'bg-primary-600 text-white rounded-br-none'
                        : 'bg-gray-100 text-gray-900 rounded-bl-none'
                      }`}
                  >
                    <p className="text-sm">{message.content}</p>
                    <div className={`text-xs mt-1 ${message.sender === 'user' ? 'text-primary-200' : 'text-gray-500'}`}>
                      {formatTime(message.timestamp)}
                    </div>
                  </div>
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>

            {/* 输入区域 */}
            <div className="border-t border-gray-200 p-4">
              <div className="flex space-x-2">
                <textarea
                  value={newMessage}
                  onChange={(e) => setNewMessage(e.target.value)}
                  onKeyDown={handleKeyPress}
                  placeholder="输入消息..."
                  className="input-field flex-1 min-h-[60px] resize-none"
                  rows={2}
                />
                <button
                  onClick={handleSendMessage}
                  className="btn-primary self-end px-6"
                  disabled={!newMessage.trim()}
                >
                  <Send className="w-4 h-4" />
                </button>
              </div>

              {/* 快速回复按钮 */}
              <div className="flex flex-wrap gap-2 mt-3">
                {['今天天气不错', '我吃了早餐', '昨晚睡得很好', '记得昨天的事情'].map((text) => (
                  <button
                    key={text}
                    onClick={() => setNewMessage(text)}
                    className="px-3 py-1.5 text-sm bg-gray-100 text-gray-700 rounded-full hover:bg-gray-200 transition-colors"
                  >
                    {text}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* 结束对话按钮 */}
          <div className="mt-6 flex justify-end">
            <button
              onClick={handleEndConversation}
              className="px-6 py-3 bg-red-600 text-white rounded-lg font-medium hover:bg-red-700 transition-colors"
            >
              结束对话并分析
            </button>
          </div>
        </div>

        {/* 侧边栏 - 分析指标 */}
        <div className="space-y-6">
          {/* 情绪评分 */}
          <div className="card">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">情绪评分</h3>
            <div className="space-y-4">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-600">当前情绪</span>
                  <span className="text-lg font-semibold">{conversationStats.moodScore}/10</span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-3">
                  <div
                    className="bg-green-500 h-3 rounded-full"
                    style={{ width: `${conversationStats.moodScore * 10}%` }}
                  />
                </div>
              </div>

              <div className="flex justify-center space-x-4">
                <button className="flex flex-col items-center p-3 hover:bg-gray-50 rounded-lg">
                  <ThumbsDown className="w-6 h-6 text-gray-400" />
                  <span className="text-xs mt-1">情绪低落</span>
                </button>
                <button className="flex flex-col items-center p-3 hover:bg-gray-50 rounded-lg">
                  <Smile className="w-6 h-6 text-yellow-400" />
                  <span className="text-xs mt-1">情绪一般</span>
                </button>
                <button className="flex flex-col items-center p-3 hover:bg-gray-50 rounded-lg">
                  <ThumbsUp className="w-6 h-6 text-green-400" />
                  <span className="text-xs mt-1">情绪良好</span>
                </button>
              </div>
            </div>
          </div>

          {/* 认知指标 */}
          <div className="card">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">认知指标</h3>
            <div className="space-y-4">
              {[
                { label: '记忆力', score: 7, color: 'bg-cognitive-memory' },
                { label: '注意力', score: 8, color: 'bg-cognitive-attention' },
                { label: '语言能力', score: 6, color: 'bg-cognitive-language' },
                { label: '执行功能', score: 7, color: 'bg-cognitive-executive' },
              ].map((item) => (
                <div key={item.label}>
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm text-gray-600">{item.label}</span>
                    <span className="text-sm font-medium">{item.score}/10</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className={`${item.color} h-2 rounded-full`}
                      style={{ width: `${item.score * 10}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 对话建议 */}
          <div className="card">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">对话建议</h3>
            <div className="space-y-3">
              <div className="p-3 bg-blue-50 rounded-lg">
                <p className="text-sm text-gray-700">
                  尝试回忆昨天发生的事情，这有助于锻炼记忆力。
                </p>
              </div>
              <div className="p-3 bg-green-50 rounded-lg">
                <p className="text-sm text-gray-700">
                  描述一个具体的场景，比如你最喜欢的公园是什么样子。
                </p>
              </div>
              <div className="p-3 bg-yellow-50 rounded-lg">
                <p className="text-sm text-gray-700">
                  分享一个有趣的故事，尽量详细描述细节。
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Conversation
