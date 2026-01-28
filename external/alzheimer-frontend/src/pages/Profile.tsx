import { useState } from 'react'
import { User, Mail, Phone, Calendar, MapPin, Book, Briefcase, Heart, Edit, Save, Lock } from 'lucide-react'

const Profile = () => {
  const [isEditing, setIsEditing] = useState(false)
  const [userData, setUserData] = useState({
    firstName: '张',
    lastName: '先生',
    email: 'zhang@example.com',
    phoneNumber: '13800138000',
    dateOfBirth: '1959-03-15',
    gender: 'male',
    educationLevel: 'university',
    occupation: '退休教师',
    familyHistory: true,
    medicalHistory: '高血压，控制良好',
    address: '北京市朝阳区',
  })

  const [riskAssessment] = useState({
    riskLevel: 'medium' as 'low' | 'medium' | 'high',
    riskScore: 4,
    riskFactors: ['age', 'family_history'],
    lastAssessment: '2024-01-10',
  })

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    const { name, value, type } = e.target
    setUserData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? (e.target as HTMLInputElement).checked : value
    }))
  }

  const handleSave = () => {
    // 这里应该调用API保存用户数据
    console.log('保存用户数据:', userData)
    setIsEditing(false)
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

  const calculateAge = (dateOfBirth: string) => {
    const birthDate = new Date(dateOfBirth)
    const today = new Date()
    let age = today.getFullYear() - birthDate.getFullYear()
    const monthDiff = today.getMonth() - birthDate.getMonth()

    if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthDate.getDate())) {
      age--
    }

    return age
  }

  const age = calculateAge(userData.dateOfBirth)

  return (
    <div className="space-y-6">
      {/* 标题和编辑按钮 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">个人资料</h1>
          <p className="text-gray-600 mt-2">管理您的个人信息和健康数据</p>
        </div>

        <button
          onClick={() => isEditing ? handleSave() : setIsEditing(true)}
          className="flex items-center px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700"
        >
          {isEditing ? (
            <>
              <Save className="w-4 h-4 mr-2" />
              保存更改
            </>
          ) : (
            <>
              <Edit className="w-4 h-4 mr-2" />
              编辑资料
            </>
          )}
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 左侧：个人信息 */}
        <div className="lg:col-span-2 space-y-6">
          {/* 基本信息卡片 */}
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">基本信息</h2>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <User className="w-4 h-4 inline mr-1" />
                  名字
                </label>
                {isEditing ? (
                  <input
                    name="firstName"
                    value={userData.firstName}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">{userData.firstName}</div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <User className="w-4 h-4 inline mr-1" />
                  姓氏
                </label>
                {isEditing ? (
                  <input
                    name="lastName"
                    value={userData.lastName}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">{userData.lastName}</div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <Mail className="w-4 h-4 inline mr-1" />
                  邮箱地址
                </label>
                {isEditing ? (
                  <input
                    name="email"
                    type="email"
                    value={userData.email}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">{userData.email}</div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <Phone className="w-4 h-4 inline mr-1" />
                  电话号码
                </label>
                {isEditing ? (
                  <input
                    name="phoneNumber"
                    type="tel"
                    value={userData.phoneNumber}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">{userData.phoneNumber}</div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <Calendar className="w-4 h-4 inline mr-1" />
                  出生日期
                </label>
                {isEditing ? (
                  <input
                    name="dateOfBirth"
                    type="date"
                    value={userData.dateOfBirth}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">
                    {userData.dateOfBirth} ({age}岁)
                  </div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  性别
                </label>
                {isEditing ? (
                  <select
                    name="gender"
                    value={userData.gender}
                    onChange={handleChange}
                    className="input-field"
                  >
                    <option value="male">男</option>
                    <option value="female">女</option>
                    <option value="other">其他</option>
                  </select>
                ) : (
                  <div className="text-gray-900 font-medium">
                    {userData.gender === 'male' ? '男' : userData.gender === 'female' ? '女' : '其他'}
                  </div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <MapPin className="w-4 h-4 inline mr-1" />
                  地址
                </label>
                {isEditing ? (
                  <input
                    name="address"
                    value={userData.address}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">{userData.address}</div>
                )}
              </div>
            </div>
          </div>

          {/* 教育和工作 */}
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">教育和工作</h2>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <Book className="w-4 h-4 inline mr-1" />
                  教育程度
                </label>
                {isEditing ? (
                  <select
                    name="educationLevel"
                    value={userData.educationLevel}
                    onChange={handleChange}
                    className="input-field"
                  >
                    <option value="low">小学及以下</option>
                    <option value="medium">中学</option>
                    <option value="high">高中</option>
                    <option value="university">大学及以上</option>
                  </select>
                ) : (
                  <div className="text-gray-900 font-medium">
                    {userData.educationLevel === 'low' ? '小学及以下' :
                     userData.educationLevel === 'medium' ? '中学' :
                     userData.educationLevel === 'high' ? '高中' : '大学及以上'}
                  </div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  <Briefcase className="w-4 h-4 inline mr-1" />
                  职业
                </label>
                {isEditing ? (
                  <input
                    name="occupation"
                    value={userData.occupation}
                    onChange={handleChange}
                    className="input-field"
                  />
                ) : (
                  <div className="text-gray-900 font-medium">{userData.occupation}</div>
                )}
              </div>
            </div>
          </div>

          {/* 健康信息 */}
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">健康信息</h2>

            <div className="space-y-6">
              <div>
                <label className="flex items-center text-sm font-medium text-gray-700 mb-2">
                  <Heart className="w-4 h-4 mr-1" />
                  家族病史
                </label>
                {isEditing ? (
                  <div className="flex items-center">
                    <input
                      id="familyHistory"
                      name="familyHistory"
                      type="checkbox"
                      checked={userData.familyHistory}
                      onChange={handleChange}
                      className="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded"
                    />
                    <label htmlFor="familyHistory" className="ml-2 text-sm text-gray-700">
                      有阿兹海默症或其他认知障碍家族史
                    </label>
                  </div>
                ) : (
                  <div className="text-gray-900 font-medium">
                    {userData.familyHistory ? '有家族病史' : '无家族病史'}
                  </div>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  其他病史
                </label>
                {isEditing ? (
                  <textarea
                    name="medicalHistory"
                    value={userData.medicalHistory}
                    onChange={handleChange}
                    className="input-field min-h-[100px]"
                    placeholder="请输入其他病史，如高血压、糖尿病等"
                  />
                ) : (
                  <div className="text-gray-900 font-medium whitespace-pre-line">
                    {userData.medicalHistory || '无'}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* 右侧：风险评估和设置 */}
        <div className="space-y-6">
          {/* 风险评估 */}
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">风险评估</h2>

            <div className="space-y-4">
              <div className="text-center">
                <div className="text-4xl font-bold text-gray-900 mb-2">
                  {riskAssessment.riskScore}/10
                </div>
                <div className="mb-4">
                  <span className={`badge ${getRiskBadgeClass(riskAssessment.riskLevel)}`}>
                    {riskAssessment.riskLevel === 'low' ? '低风险' :
                     riskAssessment.riskLevel === 'medium' ? '中风险' : '高风险'}
                  </span>
                </div>
                <p className="text-sm text-gray-500">
                  上次评估：{riskAssessment.lastAssessment}
                </p>
              </div>

              <div className="pt-4 border-t border-gray-200">
                <h3 className="text-sm font-medium text-gray-700 mb-3">风险因素</h3>
                <div className="space-y-2">
                  {riskAssessment.riskFactors.map((factor, index) => (
                    <div key={index} className="flex items-center text-sm">
                      <div className="w-2 h-2 bg-red-500 rounded-full mr-2"></div>
                      {factor === 'age' ? '年龄因素' :
                       factor === 'family_history' ? '家族病史' :
                       factor === 'gender' ? '性别因素' : '教育因素'}
                    </div>
                  ))}
                </div>
              </div>

              <button className="btn-primary w-full mt-4">
                重新评估风险
              </button>
            </div>
          </div>

          {/* 账户安全 */}
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">账户安全</h2>

            <div className="space-y-4">
              <button className="w-full flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100">
                <div className="flex items-center">
                  <Lock className="w-5 h-5 text-gray-400 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">修改密码</p>
                    <p className="text-xs text-gray-500">定期更新密码保障安全</p>
                  </div>
                </div>
                <div className="text-gray-400">→</div>
              </button>

              <button className="w-full flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100">
                <div className="flex items-center">
                  <Mail className="w-5 h-5 text-gray-400 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">邮箱验证</p>
                    <p className="text-xs text-gray-500">已验证</p>
                  </div>
                </div>
                <div className="text-green-500 text-sm">✓</div>
              </button>

              <button className="w-full flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100">
                <div className="flex items-center">
                  <Phone className="w-5 h-5 text-gray-400 mr-3" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">手机验证</p>
                    <p className="text-xs text-gray-500">已验证</p>
                  </div>
                </div>
                <div className="text-green-500 text-sm">✓</div>
              </button>
            </div>
          </div>

          {/* 数据管理 */}
          <div className="card">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">数据管理</h2>

            <div className="space-y-3">
              <button className="w-full text-left p-3 text-sm text-gray-700 bg-gray-50 rounded-lg hover:bg-gray-100">
                导出个人数据
              </button>
              <button className="w-full text-left p-3 text-sm text-gray-700 bg-gray-50 rounded-lg hover:bg-gray-100">
                查看数据使用政策
              </button>
              <button className="w-full text-left p-3 text-sm text-red-600 bg-red-50 rounded-lg hover:bg-red-100">
                删除账户
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Profile