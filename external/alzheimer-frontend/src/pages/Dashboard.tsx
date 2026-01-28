import { useState } from 'react'
import { Link } from 'react-router-dom'
import {
  TrendingUp,
  TrendingDown,
  Minus,
  Brain,
  MessageSquare,
  Activity,
  AlertTriangle,
  Calendar,
  Clock,
  Users
} from 'lucide-react'

const Dashboard = () => {
  const [stats] = useState({
    cognitiveScore: 78,
    riskLevel: 'medium' as 'low' | 'medium' | 'high',
    conversationCount: 24,
    assessmentCount: 3,
    moodTrend: 'improving' as 'improving' | 'stable' | 'declining',
    engagementTrend: 'stable' as 'improving' | 'stable' | 'declining',
  })

  const [recentConversations] = useState([
    { id: 1, title: '今天的早餐', time: '2小时前', mood: 8, engagement: 7 },
    { id: 2, title: '回忆童年', time: '昨天', mood: 9, engagement: 8 },
    { id: 3, title: '周末计划', time: '3天前', mood: 7, engagement: 6 },
  ])

  const [recommendations] = useState([
    '建议增加社交互动频率',
    '进行记忆训练游戏',
    '保持规律作息',
    '多进行户外活动',
  ])

  const getTrendIcon = (trend: 'improving' | 'stable' | 'declining') => {
    switch (trend) {
      case 'improving':
        return <TrendingUp className="w-4 h-4 text-green-500" />
      case 'declining':
        return <TrendingDown className="w-4 h-4 text-red-500" />
      default:
        return <Minus className="w-4 h-4 text-gray-500" />
    }
  }

  const getRiskBadgeClass = (level: string) => {
    switch (level) {
      case 'low':
        return 'badge-low'
      case 'medium':
        return 'badge-medium'
      case 'high':
        return 'badge-high'
      default:
        return 'badge-low'
    }
  }

  return (
    <div className="space-y-6">
      {/* 欢迎标题 */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">欢迎回来，张先生</h1>
        <p className="text-gray-600 mt-2">今天是2024年1月12日，让我们一起关注认知健康</p>
      </div>

      {/* 关键指标 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* 认知分数 */}
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">认知分数</p>
              <p className="text-3xl font-bold text-gray-900 mt-2">{stats.cognitiveScore}</p>
              <div className="flex items-center mt-2">
                {getTrendIcon(stats.moodTrend)}
                <span className="text-sm text-gray-500 ml-2">较上周 +5%</span>
              </div>
            </div>
            <div className="w-12 h-12 bg-primary-50 rounded-full flex items-center justify-center">
              <Brain className="w-6 h-6 text-primary-600" />
            </div>
          </div>
        </div>

        {/* 风险等级 */}
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">风险等级</p>
              <div className="flex items-center mt-2">
                <span className={`badge ${getRiskBadgeClass(stats.riskLevel)}`}>
                  {stats.riskLevel === 'low' ? '低风险' : stats.riskLevel === 'medium' ? '中风险' : '高风险'}
                </span>
              </div>
              <p className="text-sm text-gray-500 mt-2">基于最近评估结果</p>
            </div>
            <div className="w-12 h-12 bg-red-50 rounded-full flex items-center justify-center">
              <AlertTriangle className="w-6 h-6 text-red-600" />
            </div>
          </div>
        </div>

        {/* 对话统计 */}
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">对话次数</p>
              <p className="text-3xl font-bold text-gray-900 mt-2">{stats.conversationCount}</p>
              <div className="flex items-center mt-2">
                {getTrendIcon(stats.engagementTrend)}
                <span className="text-sm text-gray-500 ml-2">本月已完成</span>
              </div>
            </div>
            <div className="w-12 h-12 bg-green-50 rounded-full flex items-center justify-center">
              <MessageSquare className="w-6 h-6 text-green-600" />
            </div>
          </div>
        </div>

        {/* 评估次数 */}
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">评估次数</p>
              <p className="text-3xl font-bold text-gray-900 mt-2">{stats.assessmentCount}</p>
              <p className="text-sm text-gray-500 mt-2">上次评估：2周前</p>
            </div>
            <div className="w-12 h-12 bg-purple-50 rounded-full flex items-center justify-center">
              <Activity className="w-6 h-6 text-purple-600" />
            </div>
          </div>
        </div>
      </div>

      {/* 主要内容区域 */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 最近对话 */}
        <div className="lg:col-span-2">
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold text-gray-900">最近对话</h2>
              <Link to="/conversation" className="text-primary-600 hover:text-primary-700 text-sm font-medium">
                查看全部 →
              </Link>
            </div>

            <div className="space-y-4">
              {recentConversations.map((conv) => (
                <div
                  key={conv.id}
                  className="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
                >
                  <div className="flex items-center space-x-4">
                    <div className="w-10 h-10 bg-white rounded-lg flex items-center justify-center">
                      <MessageSquare className="w-5 h-5 text-gray-400" />
                    </div>
                    <div>
                      <h3 className="font-medium text-gray-900">{conv.title}</h3>
                      <div className="flex items-center text-sm text-gray-500 mt-1">
                        <Clock className="w-3 h-3 mr-1" />
                        {conv.time}
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center space-x-4">
                    <div className="text-center">
                      <div className="text-xs text-gray-500">情绪</div>
                      <div className="flex items-center">
                        <div className="w-16 bg-gray-200 rounded-full h-2">
                          <div
                            className="bg-green-500 h-2 rounded-full"
                            style={{ width: `${conv.mood * 10}%` }}
                          />
                        </div>
                        <span className="ml-2 text-sm font-medium">{conv.mood}/10</span>
                      </div>
                    </div>

                    <div className="text-center">
                      <div className="text-xs text-gray-500">参与度</div>
                      <div className="flex items-center">
                        <div className="w-16 bg-gray-200 rounded-full h-2">
                          <div
                            className="bg-blue-500 h-2 rounded-full"
                            style={{ width: `${conv.engagement * 10}%` }}
                          />
                        </div>
                        <span className="ml-2 text-sm font-medium">{conv.engagement}/10</span>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            <div className="mt-6 pt-6 border-t border-gray-200">
              <Link
                to="/conversation"
                className="btn-primary w-full flex items-center justify-center"
              >
                <MessageSquare className="w-4 h-4 mr-2" />
                开始新对话
              </Link>
            </div>
          </div>
        </div>

        {/* 建议和提醒 */}
        <div>
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold text-gray-900">个性化建议</h2>
              <Users className="w-5 h-5 text-gray-400" />
            </div>

            <div className="space-y-4">
              {recommendations.map((rec, index) => (
                <div
                  key={index}
                  className="flex items-start space-x-3 p-3 bg-blue-50 rounded-lg"
                >
                  <div className="w-6 h-6 bg-primary-100 rounded-full flex items-center justify-center flex-shrink-0">
                    <span className="text-xs font-medium text-primary-700">{index + 1}</span>
                  </div>
                  <p className="text-sm text-gray-700">{rec}</p>
                </div>
              ))}
            </div>
          </div>

          {/* 提醒卡片 */}
          <div className="card mt-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-gray-900">提醒</h2>
              <Calendar className="w-5 h-5 text-gray-400" />
            </div>

            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-yellow-50 rounded-lg">
                <div className="flex items-center space-x-3">
                  <div className="w-8 h-8 bg-yellow-100 rounded-full flex items-center justify-center">
                    <Brain className="w-4 h-4 text-yellow-600" />
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">认知评估</p>
                    <p className="text-xs text-gray-500">建议每月进行一次</p>
                  </div>
                </div>
                <button className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                  开始
                </button>
              </div>

              <div className="flex items-center justify-between p-3 bg-green-50 rounded-lg">
                <div className="flex items-center space-x-3">
                  <div className="w-8 h-8 bg-green-100 rounded-full flex items-center justify-center">
                    <MessageSquare className="w-4 h-4 text-green-600" />
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">日常对话</p>
                    <p className="text-xs text-gray-500">今天还未进行对话</p>
                  </div>
                </div>
                <button className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                  开始
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Dashboard