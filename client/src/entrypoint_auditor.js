'use strict'

const m = require('mithril')

const { App, Welcome } = require('App/views/auditor')
const { AgentProfile } = require('App/views/common/profile')
const { SignInForm, AgentSignUpForm } = require('App/views/common/auth')
const { CreateCertifyingBody } = require('App/views/auditor/organization')
const { FactoryList } = require('App/views/auditor/factory')
const { RequestList } = require('App/views/auditor/requests')
const { CertificateCreate, CertificateList } = require('App/views/auditor/certificates')
const AuthService = require('App/services/auth')

AuthService.namespace = 'auditor'

let element = document.getElementById("app")
m.route(element, '/', {
  '/': App.subpage(Welcome),

  '/signIn': App.subpage(SignInForm),
  '/signUp': App.subpage(AgentSignUpForm),
  '/profile': App.subpage(AgentProfile),

  '/organizationCreate': App.subpage(CreateCertifyingBody),

  '/requests': App.subpage(RequestList),
  '/factories': App.subpage(FactoryList),

  '/certificateCreate': App.subpage(CertificateCreate),
  '/certificates': App.subpage(CertificateList),
})
