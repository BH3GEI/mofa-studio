import { useState } from 'react'
import { TrendingUp, TrendingDown, Minus, Calendar, Filter, Download, Eye, Brain, MessageSquare, Activity, Users } from 'lucide-react'

const Analytics = () => {
  const [timeRange, setTimeRange] = useState<'week' | 'month' | 'quarter' | 'year'>('month')

  // 模拟数据
  const [analyticsData] = useState({
    overview: {
      totalConversations: 24,
      totalAssessments: 3,
      avgCognitiveScore: 78,
      riskLevel: 'medium' as 'low' | 'medium' | 'high',
    },
    trends: {
      cognitiveScores: [
        { date: '1月1日', score: 75 },
        { date: '1月8日', score: 78 },
        { date: '1月15日', score: 76 },
        { date: '1月22日', score: 80 },
        { date: '1月29日', score: 82 },
        { date: '2月5日', score: 78 },
      ],
      conversationFrequency: [
        { date: '1月', count: 5 },
        { date: '2月', count: 8 },
        { date: '3月', count: 6 },
        { date: '4月', count: 5 },
      ],
      domainScores: [
        { domain: '记忆力', score: 7.2, trend: 'improving' as 'improving' | 'stable' | 'declining' },
        { domain: '注意力', score: 8.1, trend: 'stable' as 'improving' | 'stable' | 'declining' },
        { domain: '语言能力', score: 6.8, trend: 'declining' as 'improving' | 'stable' | 'declining' },
        { domain: '执行功能', score: 7.5, trend: 'improving' as 'improving' | 'stable' | 'declining' },
        { domain: '视空间', score: 7.9, trend: 'stable' as 'improving' | 'stable' | 'declining' },
      ],
    },
    insights: [
      '记忆力在最近一个月有明显改善',
      '语言能力略有下降，建议加强语言训练',
      '对话频率保持稳定，参与度良好',
      '情绪状态整体积极，有助于认知健康',
    ],
    recommendations: [
      '继续坚持记忆训练游戏',
      '增加语言交流活动，如朗读、对话',
      '保持每周3次以上的社交互动',
      '定期进行认知评估，跟踪变化',
    ],
  })

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

  const getDomainColor = (domain: string) => {
    switch (domain) {
      case '记忆力':
        return 'bg-purple-100 text-purple-800'
      case '注意力':
        return 'bg-green-100 text-green-800'
      case '语言能力':
        return 'bg-yellow-100 text-yellow-800'
      case '执行功能':
        return 'bg-blue-100 text-blue-800'
      case '视空间':
        return 'bg-red-100 text-red-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="space-y-6">
      {/* 标题和筛选器 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">数据分析</h1>
          <p className="text-gray-600 mt-2">深入了解认知健康趋势和模式</p>
        </div>

        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <Calendar className="w-5 h-5 text-gray-400" />
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as any)}
              className="input-field py-1.5 text-sm"
            >
              <option value="week">最近一周</option>
              <option value="month">最近一月</option>
              <option value="quarter">最近一季度</option>
              <option value="year">最近一年</option>
            </select>
          </div>

          <button className="flex items-center px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50">
            <Filter className="w-4 h-4 mr-2" />
            筛选
          </button>

          <button className="flex items-center px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700">
            <Download className="w-4 h-4 mr-2" />
            导出报告
          </button>
        </div>
      </div>

      {/* 概览卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">对话总数</p>
              <p className="text-3xl font-bold text-gray-900 mt-2">
                {analyticsData.overview.totalConversations}
              </p>
              <div className="flex items-center mt-2">
                <TrendingUp className="w-4 h-4 text-green-500" />
                <span className="text-sm text-gray-500 ml-2">较上月 +20%</span>
              </div>
            </div>
            <div className="w-12 h-12 bg-blue-50 rounded-full flex items-center justify-center">
              <MessageSquare className="w-6 h-6 text-blue-600" />
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">评估次数</p>
              <p className="text-3xl font-bold text-gray-900 mt-2">
                {analyticsData.overview.totalAssessments}
              </p>
              <p className="text-sm text-gray-500 mt-2">上次评估：2周前</p>
            </div>
            <div className="w-12 h-12 bg-purple-50 rounded-full flex items-center justify-center">
              <Brain className="w-6 h-6 text-purple-600" />
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">平均认知分数</p>
              <p className="text-3xl font-bold text-gray-900 mt-2">
                {analyticsData.overview.avgCognitiveScore}
              </p>
              <div className="flex items-center mt-2">
                <TrendingUp className="w-4 h-4 text-green-500" />
                <span className="text-sm text-gray-500 ml-2">较上月 +5%</span>
              </div>
            </div>
            <div className="w-12 h-12 bg-green-50 rounded-full flex items-center justify-center">
              <Activity className="w-6 h-6 text-green-600" />
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">风险等级</p>
              <div className="flex items-center mt-2">
                <span className={`badge ${getRiskBadgeClass(analyticsData.overview.riskLevel)}`}>
                  {analyticsData.overview.riskLevel === 'low' ? '低风险' :
                    analyticsData.overview.riskLevel === 'medium' ? '中风险' : '高风险'}
                </span>
              </div>
              <p className="text-sm text-gray-500 mt-2">基于最近评估结果</p>
            </div>
            <div className="w-12 h-12 bg-red-50 rounded-full flex items-center justify-center">
              <Eye className="w-6 h-6 text-red-600" />
            </div>
          </div>
        </div>
      </div>

      {/* 主要内容区域 */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 认知分数趋势 */}
        <div className="lg:col-span-2">
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold text-gray-900">认知分数趋势</h2>
              <div className="flex items-center text-sm text-gray-500">
                <Calendar className="w-4 h-4 mr-1" />
                {timeRange === 'week' ? '最近一周' :
                  timeRange === 'month' ? '最近一月' :
                    timeRange === 'quarter' ? '最近一季度' : '最近一年'}
              </div>
            </div>

            <div className="h-64 flex items-center justify-center bg-gray-50 rounded-lg">
              <div className="text-center">
                <div className="text-lg font-semibold text-gray-900 mb-2">趋势图表</div>
                <p className="text-gray-600">这里显示认知分数随时间的变化趋势</p>
                <div className="mt-4 flex justify-center space-x-2">
                  {analyticsData.trends.cognitiveScores.map((item, index) => (
                    <div key={index} className="flex flex-col items-center">
                      <div
                        className="w-8 bg-primary-600 rounded-t"
                        style={{ height: `${item.score * 2}px` }}
                      />
                      <div className="text-xs text-gray-500 mt-1">{item.date}</div>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            <div className="mt-6 grid grid-cols-2 gap-4">
              <div className="p-4 bg-blue-50 rounded-lg">
                <div className="text-sm text-gray-600">最高分数</div>
                <div className="text-2xl font-bold text-gray-900 mt-1">82</div>
                <div className="text-xs text-gray-500">1月29日</div>
              </div>
              <div className="p-4 bg-green-50 rounded-lg">
                <div className="text-sm text-gray-600">平均分数</div>
                <div className="text-2xl font-bold text-gray-900 mt-1">78.2</div>
                <div className="text-xs text-gray-500">整体表现良好</div>
              </div>
            </div>
          </div>
        </div>

        {/* 各领域表现 */}
        <div>
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">各领域表现</h2>

            <div className="space-y-4">
              {analyticsData.trends.domainScores.map((item) => (
                <div key={item.domain} className="p-4 bg-gray-50 rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center">
                      <span className={`badge ${getDomainColor(item.domain)}`}>
                        {item.domain}
                      </span>
                      <span className="ml-2 text-sm text-gray-500">
                        {item.score}/10
                      </span>
                    </div>
                    {getTrendIcon(item.trend)}
                  </div>

                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className={`h-2 rounded-full ${getDomainColor(item.domain).split(' ')[0]}`}
                      style={{ width: `${item.score * 10}%` }}
                    />
                  </div>

                  <div className="mt-2 text-xs text-gray-500">
                    {item.trend === 'improving' ? '趋势改善中' :
                      item.trend === 'declining' ? '趋势下降中' : '趋势稳定'}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* 对话频率和洞察 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 对话频率 */}
        <div className="card">
          <h2 className="text-lg font-semibold text-gray-900 mb-6">对话频率</h2>

          <div className="h-48 flex items-center justify-center bg-gray-50 rounded-lg mb-6">
            <div className="text-center">
              <div className="text-lg font-semibold text-gray-900 mb-2">月度对话统计</div>
              <div className="mt-4 flex justify-center space-x-4">
                {analyticsData.trends.conversationFrequency.map((item, index) => (
                  <div key={index} className="flex flex-col items-center">
                    <div
                      className="w-12 bg-green-600 rounded-t"
                      style={{ height: `${item.count * 10}px` }}
                    />
                    <div className="text-sm font-medium text-gray-900 mt-2">{item.count}</div>
                    <div className="text-xs text-gray-500">{item.date}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 bg-blue-50 rounded-lg">
              <div className="flex items-center">
                <MessageSquare className="w-5 h-5 text-blue-600 mr-3" />
                <div>
                  <p className="text-sm font-medium text-gray-900">平均每周对话</p>
                  <p className="text-xs text-gray-500">保持良好互动频率</p>
                </div>
              </div>
              <div className="text-lg font-semibold">2.4次</div>
            </div>

            <div className="flex items-center justify-between p-3 bg-green-50 rounded-lg">
              <div className="flex items-center">
                <Users className="w-5 h-5 text-green-600 mr-3" />
                <div>
                  <p className="text-sm font-medium text-gray-900">最佳对话时段</p>
                  <p className="text-xs text-gray-500">上午9-11点</p>
                </div>
              </div>
              <div className="text-lg font-semibold">78%参与度</div>
            </div>
          </div>
        </div>

        {/* 洞察和建议 */}
        <div className="card">
          <h2 className="text-lg font-semibold text-gray-900 mb-6">洞察与建议</h2>

          <div className="space-y-6">
            {/* 洞察 */}
            <div>
              <h3 className="text-md font-medium text-gray-900 mb-3">关键洞察</h3>
              <div className="space-y-3">
                {analyticsData.insights.map((insight, index) => (
                  <div key={index} className="flex items-start p-3 bg-gray-50 rounded-lg">
                    <div className="w-6 h-6 bg-primary-100 rounded-full flex items-center justify-center flex-shrink-0 mr-3">
                      <Eye className="w-3 h-3 text-primary-700" />
                    </div>
                    <p className="text-sm text-gray-700">{insight}</p>
                  </div>
                ))}
              </div>
            </div>

            {/* 建议 */}
            <div>
              <h3 className="text-md font-medium text-gray-900 mb-3">优化建议</h3>
              <div className="space-y-3">
                {analyticsData.recommendations.map((recommendation, index) => (
                  <div key={index} className="flex items-start p-3 bg-blue-50 rounded-lg">
                    <div className="w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center flex-shrink-0 mr-3">
                      <span className="text-xs font-medium text-blue-700">{index + 1}</span>
                    </div>
                    <p className="text-sm text-gray-700">{recommendation}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 风险评估历史 */}
      <div className="card">
        <h2 className="text-lg font-semibold text-gray-900 mb-6">风险评估历史</h2>

        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-200">
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">评估日期</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">评估类型</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">总分</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">百分比</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">风险等级</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">趋势</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-gray-600">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-gray-100 hover:bg-gray-50">
                <td className="py-3 px-4 text-sm">2024-01-10</td>
                <td className="py-3 px-4 text-sm">快速评估</td>
                <td className="py-3 px-4 text-sm font-medium">28/30</td>
                <td className="py-3 px-4 text-sm">93%</td>
                <td className="py-3 px-4">
                  <span className="badge badge-low">低风险</span>
                </td>
                <td className="py-3 px-4">
                  <div className="flex items-center">
                    <TrendingUp className="w-4 h-4 text-green-500" />
                    <span className="text-sm text-gray-600 ml-1">改善</span>
                  </div>
                </td>
                <td className="py-3 px-4">
                  <button className="text-primary-600 hover:text-primary-700 text-sm font-medium">
                    查看详情
                  </button>
                </td>
              </tr>
              <tr className="border-b border-gray-100 hover:bg-gray-50">
                <td className="py-3 px-4 text-sm">2023-12-15</td>
                <td className="py-3 px-4 text-sm">MMSE评估</td>
                <td className="py-3 px-4 text-sm font-medium">26/30</td>
                <td className="py-3 px-4 text-sm">87%</td>
                <td className="py-3 px-4">
                  <span className="badge badge-low">低风险</span>
                </td>
                <td className="py-3 px-4">
                  <div className="flex items-center">
                    <Minus className="w-4 h-4 text-gray-500" />
                    <span className="text-sm text-gray-600 ml-1">稳定</span>
                  </div>
                </td>
                <td className="py-3 px-4">
                  <button className="text-primary-600 hover:text-primary-700 text-sm font-medium">
                    查看详情
                  </button>
                </td>
              </tr>
              <tr className="hover:bg-gray-50">
                <td className="py-3 px-4 text-sm">2023-11-20</td>
                <td className="py-3 px-4 text-sm">快速评估</td>
                <td className="py-3 px-4 text-sm font-medium">24/30</td>
                <td className="py-3 px-4 text-sm">80%</td>
                <td className="py-3 px-4">
                  <span className="badge badge-medium">中风险</span>
                </td>
                <td className="py-3 px-4">
                  <div className="flex items-center">
                    <TrendingDown className="w-4 h-4 text-red-500" />
                    <span className="text-sm text-gray-600 ml-1">下降</span>
                  </div>
                </td>
                <td className="py-3 px-4">
                  <button className="text-primary-600 hover:text-primary-700 text-sm font-medium">
                    查看详情
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}

export default Analytics