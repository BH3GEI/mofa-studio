import { useState } from 'react'
import { Brain, Clock, CheckCircle, TrendingUp, BarChart3 } from 'lucide-react'

interface AssessmentQuestion {
  id: string
  question: string
  type: 'orientation' | 'memory' | 'attention' | 'language' | 'visuospatial'
  options?: string[]
  answer?: string | number
  score: number
  maxScore: number
}

const Assessment = () => {
  const [currentStep, setCurrentStep] = useState<'intro' | 'questions' | 'results'>('intro')
  const [assessmentType, setAssessmentType] = useState<'quick' | 'mmse' | 'moca'>('quick')
  const [questions, setQuestions] = useState<AssessmentQuestion[]>([
    {
      id: 'orientation_time',
      question: '今天是几月几日？星期几？',
      type: 'orientation',
      score: 0,
      maxScore: 5,
    },
    {
      id: 'orientation_place',
      question: '我们现在在哪里？（城市、区、地点）',
      type: 'orientation',
      score: 0,
      maxScore: 5,
    },
    {
      id: 'memory_immediate',
      question: '请记住这三个词：苹果、桌子、红色',
      type: 'memory',
      score: 0,
      maxScore: 3,
    },
    {
      id: 'attention',
      question: '从100开始，每次减7，连续减5次',
      type: 'attention',
      score: 0,
      maxScore: 5,
    },
    {
      id: 'memory_delayed',
      question: '刚才让你记住的三个词是什么？',
      type: 'memory',
      score: 0,
      maxScore: 3,
    },
    {
      id: 'language',
      question: '请说出一分钟之内你能想到的所有动物',
      type: 'language',
      score: 0,
      maxScore: 5,
    },
    {
      id: 'visuospatial',
      question: '请画一个时钟，指针指向10点10分',
      type: 'visuospatial',
      score: 0,
      maxScore: 4,
    },
  ])

  const [currentQuestionIndex, setCurrentQuestionIndex] = useState(0)
  const [userAnswers, setUserAnswers] = useState<Record<string, any>>({})
  const [results, setResults] = useState({
    totalScore: 0,
    maxScore: 30,
    percentage: 0,
    riskLevel: 'low' as 'low' | 'medium' | 'high',
    domainScores: {
      orientation: 0,
      memory: 0,
      attention: 0,
      language: 0,
      visuospatial: 0,
    },
  })

  const handleStartAssessment = (type: 'quick' | 'mmse' | 'moca') => {
    setAssessmentType(type)
    setCurrentStep('questions')
    // 这里可以根据类型加载不同的问题集
  }

  const handleAnswer = (questionId: string, answer: any, score: number) => {
    const updatedAnswers = { ...userAnswers, [questionId]: { answer, score } }
    setUserAnswers(updatedAnswers)

    // 更新问题分数
    const updatedQuestions = questions.map(q =>
      q.id === questionId ? { ...q, score, answer } : q
    )
    setQuestions(updatedQuestions)

    // 自动进入下一题或显示结果
    if (currentQuestionIndex < questions.length - 1) {
      setCurrentQuestionIndex(currentQuestionIndex + 1)
    } else {
      calculateResults(updatedQuestions)
    }
  }

  const calculateResults = (questions: AssessmentQuestion[]) => {
    const totalScore = questions.reduce((sum, q) => sum + q.score, 0)
    const maxScore = questions.reduce((sum, q) => sum + q.maxScore, 0)
    const percentage = Math.round((totalScore / maxScore) * 100)

    // 计算各领域分数
    const domainScores = {
      orientation: 0,
      memory: 0,
      attention: 0,
      language: 0,
      visuospatial: 0,
    }

    questions.forEach(q => {
      domainScores[q.type] += q.score
    })

    // 确定风险等级
    let riskLevel: 'low' | 'medium' | 'high' = 'low'
    if (percentage < 60) {
      riskLevel = 'high'
    } else if (percentage < 80) {
      riskLevel = 'medium'
    }

    setResults({
      totalScore,
      maxScore,
      percentage,
      riskLevel,
      domainScores,
    })

    setCurrentStep('results')
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
      case 'orientation':
        return 'bg-blue-100 text-blue-800'
      case 'memory':
        return 'bg-purple-100 text-purple-800'
      case 'attention':
        return 'bg-green-100 text-green-800'
      case 'language':
        return 'bg-yellow-100 text-yellow-800'
      case 'visuospatial':
        return 'bg-red-100 text-red-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  const getAssessmentTypeLabel = (type: 'quick' | 'mmse' | 'moca') => {
    switch (type) {
      case 'quick':
        return '快速评估'
      case 'mmse':
        return 'MMSE评估'
      case 'moca':
        return 'MoCA评估'
      default:
        return '快速评估'
    }
  }

  const currentQuestion = questions[currentQuestionIndex]

  return (
    <div className="space-y-6">
      {/* 标题 */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">认知评估</h1>
        <p className="text-gray-600 mt-2">定期评估认知功能，及早发现变化</p>
      </div>

      {currentStep === 'intro' && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* 快速评估 */}
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-full flex items-center justify-center">
                <Clock className="w-6 h-6 text-blue-600" />
              </div>
              <span className="badge badge-low">5-10分钟</span>
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">快速评估</h3>
            <p className="text-gray-600 text-sm mb-4">
              包含7个核心问题，快速了解基本认知状态
            </p>
            <button
              onClick={() => handleStartAssessment('quick')}
              className="btn-primary w-full"
            >
              开始快速评估
            </button>
          </div>

          {/* MMSE评估 */}
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-purple-50 rounded-full flex items-center justify-center">
                <Brain className="w-6 h-6 text-purple-600" />
              </div>
              <span className="badge badge-medium">10-15分钟</span>
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">MMSE评估</h3>
            <p className="text-gray-600 text-sm mb-4">
              简易精神状态检查，标准化认知功能筛查
            </p>
            <button
              onClick={() => handleStartAssessment('mmse')}
              className="btn-secondary w-full"
            >
              开始MMSE评估
            </button>
          </div>

          {/* MoCA评估 */}
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-full flex items-center justify-center">
                <BarChart3 className="w-6 h-6 text-green-600" />
              </div>
              <span className="badge badge-medium">15-20分钟</span>
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">MoCA评估</h3>
            <p className="text-gray-600 text-sm mb-4">
              蒙特利尔认知评估，检测轻度认知障碍
            </p>
            <button
              onClick={() => handleStartAssessment('moca')}
              className="btn-secondary w-full"
            >
              开始MoCA评估
            </button>
          </div>
        </div>
      )}

      {currentStep === 'questions' && (
        <div className="card">
          {/* 进度条 */}
          <div className="mb-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-gray-600">
                问题 {currentQuestionIndex + 1} / {questions.length}
              </span>
              <span className="text-sm text-gray-600">
                类型：{getAssessmentTypeLabel(assessmentType)}
              </span>
              <span className="text-sm font-medium">
                已得分数：{questions.reduce((sum, q) => sum + q.score, 0)} / {questions.reduce((sum, q) => sum + q.maxScore, 0)}
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-primary-600 h-2 rounded-full"
                style={{ width: `${((currentQuestionIndex + 1) / questions.length) * 100}%` }}
              />
            </div>
          </div>

          {/* 当前问题 */}
          <div className="mb-8">
            <div className="flex items-center mb-4">
              <span className={`badge ${getDomainColor(currentQuestion.type)}`}>
                {currentQuestion.type === 'orientation' ? '定向力' :
                  currentQuestion.type === 'memory' ? '记忆力' :
                    currentQuestion.type === 'attention' ? '注意力' :
                      currentQuestion.type === 'language' ? '语言能力' : '视空间能力'}
              </span>
              <span className="ml-2 text-sm text-gray-500">
                分值：{currentQuestion.maxScore}分
              </span>
            </div>

            <h3 className="text-xl font-semibold text-gray-900 mb-6">
              {currentQuestion.question}
            </h3>

            {/* 答案输入区域 */}
            <div className="space-y-4">
              {currentQuestion.type === 'attention' ? (
                <div className="space-y-3">
                  <p className="text-gray-600">请依次说出计算结果：</p>
                  <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
                    {[100, 93, 86, 79, 72].map((num, index) => (
                      <input
                        key={index}
                        type="number"
                        placeholder={`${index === 0 ? '100' : ''}`}
                        className="input-field text-center"
                        onChange={(e) => {
                          const value = parseInt(e.target.value)
                          if (!isNaN(value)) {
                            // 简单的评分逻辑
                            const score = value === num ? 1 : 0
                            handleAnswer(currentQuestion.id, value, score)
                          }
                        }}
                      />
                    ))}
                  </div>
                </div>
              ) : currentQuestion.type === 'language' ? (
                <div>
                  <textarea
                    placeholder="输入你能想到的动物名称，用逗号分隔..."
                    className="input-field min-h-[100px]"
                    onChange={(e) => {
                      const animals = e.target.value.split(/[,，\s]+/).filter(a => a.trim())
                      const score = Math.min(animals.length, 5) // 最多5分
                      handleAnswer(currentQuestion.id, animals, score)
                    }}
                  />
                  <p className="text-sm text-gray-500 mt-2">想到多少就写多少，每个动物1分，最多5分</p>
                </div>
              ) : (
                <div>
                  <input
                    type="text"
                    placeholder="输入你的答案..."
                    className="input-field"
                    onChange={(e) => {
                      const answer = e.target.value
                      // 简单的评分逻辑（实际应该更复杂）
                      const score = answer.length > 0 ? currentQuestion.maxScore : 0
                      handleAnswer(currentQuestion.id, answer, score)
                    }}
                  />
                </div>
              )}
            </div>
          </div>

          {/* 导航按钮 */}
          <div className="flex justify-between">
            <button
              onClick={() => setCurrentQuestionIndex(Math.max(0, currentQuestionIndex - 1))}
              className="btn-secondary"
              disabled={currentQuestionIndex === 0}
            >
              上一题
            </button>

            <div className="flex space-x-3">
              <button
                onClick={() => handleAnswer(currentQuestion.id, '', 0)}
                className="px-4 py-2 text-gray-600 hover:text-gray-800"
              >
                跳过
              </button>
              {currentQuestionIndex < questions.length - 1 ? (
                <button
                  onClick={() => setCurrentQuestionIndex(currentQuestionIndex + 1)}
                  className="btn-primary"
                >
                  下一题
                </button>
              ) : (
                <button
                  onClick={() => calculateResults(questions)}
                  className="btn-primary"
                >
                  完成评估
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {currentStep === 'results' && (
        <div className="space-y-6">
          {/* 结果概览 */}
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h2 className="text-2xl font-bold text-gray-900">评估结果</h2>
                <p className="text-gray-600">评估完成时间：{new Date().toLocaleDateString('zh-CN')}</p>
              </div>
              <div className="text-right">
                <div className="text-4xl font-bold text-gray-900">{results.percentage}%</div>
                <div className="flex items-center justify-end mt-2">
                  <span className={`badge ${getRiskBadgeClass(results.riskLevel)}`}>
                    {results.riskLevel === 'low' ? '低风险' :
                      results.riskLevel === 'medium' ? '中风险' : '高风险'}
                  </span>
                </div>
              </div>
            </div>

            {/* 分数详情 */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-4">总分</h3>
                <div className="flex items-center justify-center">
                  <div className="relative w-48 h-48">
                    <svg className="w-full h-full" viewBox="0 0 100 100">
                      <circle
                        cx="50"
                        cy="50"
                        r="45"
                        fill="none"
                        stroke="#e5e7eb"
                        strokeWidth="10"
                      />
                      <circle
                        cx="50"
                        cy="50"
                        r="45"
                        fill="none"
                        stroke="#3b82f6"
                        strokeWidth="10"
                        strokeLinecap="round"
                        strokeDasharray={`${results.percentage * 2.83} 283`}
                        transform="rotate(-90 50 50)"
                      />
                    </svg>
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                      <div className="text-3xl font-bold">{results.totalScore}</div>
                      <div className="text-gray-500">/{results.maxScore}分</div>
                    </div>
                  </div>
                </div>
              </div>

              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-4">各领域分数</h3>
                <div className="space-y-4">
                  {Object.entries(results.domainScores).map(([domain, score]) => (
                    <div key={domain}>
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-sm text-gray-600">
                          {domain === 'orientation' ? '定向力' :
                            domain === 'memory' ? '记忆力' :
                              domain === 'attention' ? '注意力' :
                                domain === 'language' ? '语言能力' : '视空间能力'}
                        </span>
                        <span className="text-sm font-medium">{score}分</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full ${getDomainColor(domain).split(' ')[0]}`}
                          style={{ width: `${(score / 10) * 100}%` }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* 建议和下一步 */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="card">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">
                <CheckCircle className="w-5 h-5 inline mr-2 text-green-500" />
                个性化建议
              </h3>
              <div className="space-y-3">
                {results.riskLevel === 'high' ? (
                  <>
                    <p className="text-gray-700">建议咨询专业医生进行详细评估</p>
                    <p className="text-gray-700">进行系统的认知康复训练</p>
                    <p className="text-gray-700">家人需要加强关注和支持</p>
                  </>
                ) : results.riskLevel === 'medium' ? (
                  <>
                    <p className="text-gray-700">建议进行认知训练游戏</p>
                    <p className="text-gray-700">保持社交活动，每周至少3次</p>
                    <p className="text-gray-700">进行有氧运动，每周150分钟</p>
                  </>
                ) : (
                  <>
                    <p className="text-gray-700">认知状态良好，继续保持</p>
                    <p className="text-gray-700">建议定期进行认知评估</p>
                    <p className="text-gray-700">保持健康的生活习惯</p>
                  </>
                )}
              </div>
            </div>

            <div className="card">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">
                <TrendingUp className="w-5 h-5 inline mr-2 text-blue-500" />
                下一步行动
              </h3>
              <div className="space-y-4">
                <button
                  onClick={() => {
                    setCurrentStep('intro')
                    setCurrentQuestionIndex(0)
                    setUserAnswers({})
                  }}
                  className="btn-primary w-full"
                >
                  重新评估
                </button>
                <button className="btn-secondary w-full">
                  查看详细报告
                </button>
                <button className="btn-secondary w-full">
                  设置提醒
                </button>
              </div>
            </div>
          </div>

          {/* 历史对比 */}
          <div className="card">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">历史趋势</h3>
            <div className="flex items-center justify-center h-48 bg-gray-50 rounded-lg">
              <div className="text-center">
                <BarChart3 className="w-12 h-12 text-gray-400 mx-auto mb-3" />
                <p className="text-gray-600">暂无历史数据</p>
                <p className="text-sm text-gray-500 mt-1">完成更多评估后查看趋势</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default Assessment
