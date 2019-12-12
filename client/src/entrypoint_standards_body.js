'use strict'

const m = require('mithril')

const { App, Welcome } = require('App/views/standards_body')
const { AgentProfile } = require('App/views/common/profile')
const { SignInForm, AgentSignUpForm } = require('App/views/common/auth')
const { CreateStandardsBody } = require('App/views/standards_body/organization')
const { StandardCreate, StandardList, StandardUpdate } = require('App/views/standards_body/standard')
const { CertifyingBodyList } = require('App/views/standards_body/certifying_body')
const { AccreditCertifyingBody } = require('App/views/standards_body/accreditations')
const AuthService = require('App/services/auth')

AuthService.namespace = 'standards_body'

let element = document.getElementById("app")
m.route(element, '/', {
  '/': App.subpage(Welcome),

  '/signIn': App.subpage(SignInForm),
  '/signUp': App.subpage(AgentSignUpForm),
  '/profile': App.subpage(AgentProfile),

  '/organizationCreate': App.subpage(CreateStandardsBody),
  '/standardsCreate': App.subpage(StandardCreate),
  '/standardsList': App.subpage(StandardList),
  '/standardsUpdate': App.subpage(StandardUpdate),

  '/certifyingBodyList': App.subpage(CertifyingBodyList),
  '/accreditCertifyingBody': App.subpage(AccreditCertifyingBody),

})
