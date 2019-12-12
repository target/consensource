'use strict'

const m = require('mithril')
const { inputField } = require('App/components/forms')
const AuthService = require('App/services/auth')
const agentService = require('App/services/agent')


const AuthedComponent = {
  view(vnode) {
    if (AuthService.isSignedIn()) {
      return vnode.children
    } else {
      return []
    }
  }
}

const SignIn = {
  submitting: false,
  errorMsg: null,

  username: "",
  password: "",

  setUsername: (value) => {
    SignIn.username = value
  },

  setPassword: (value) => {
    SignIn.password = value
  },

  clear: () => {
    SignIn.submitting = false
    SignIn.errorMsg = null
    SignIn.username = ''
    SignIn.password = ''
  },

  submit: () => {
    SignIn.submitting = true
    AuthService.authenticate(SignIn.username, SignIn.password)
      .then(() => {
        SignIn.clear()
        m.route.set('/')
      })
      .catch((e) => {
        console.error(e)
        SignIn.errorMsg = e
        SignIn.submitting = false
      })
  }
}


/**
 * Form for Signing in a User
 */
const SignInForm = {
  oninit() {
    SignIn.clear()
  },

  view() {
    return [
      m('h2', 'Sign In'),
      m('.form', [
        SignIn.errorMsg ? m('p.text-danger', SignIn.errorMsg) : null,
        inputField('username', 'Email', SignIn.username, SignIn.setUsername),
        inputField('password', 'Password', SignIn.password, SignIn.setPassword, 'password'),
        m('.row', [
          m("button.btn.btn-primary",
            {
              onclick: SignIn.submit,
              disabled: SignIn.submitting,
            }, "Sign In"),
          m('a.btn.btn-link.small.text-muted[href=/signUp]', {
            oncreate: m.route.link
          }, 'Not a member? Sign Up')
        ])
      ]),
    ]
  }
}

const AgentSignUp = {
  submitting: false,
  errorMsg: null,

  username: '',
  password: '',
  confirmPassword: '',
  name: "",

  setUsername: (value) => {
    AgentSignUp.username = value
  },

  setPassword: (value) => {
    AgentSignUp.password = value
  },

  setConfirmPassword: (value) => {
    AgentSignUp.confirmPassword = value
  },

  setName: (value) => {
    AgentSignUp.name = value
  },

  submit: () => {
    AgentSignUp.submitting = true
    AuthService.createUser(AgentSignUp, (signer) => agentService.createAgent(AgentSignUp.name, signer))
      .then(() => {
        AgentSignUp.clear()
        m.route.set('/')
      })
      .catch((e) => {
        console.error(e)
        AgentSignUp.submitting = false
        AgentSignUp.errorMsg = e
      })

  },

  clear: () => {
    AgentSignUp.submitting = false
    AgentSignUp.errorMsg = null

    AgentSignUp.username = ''
    AgentSignUp.password = ''
    AgentSignUp.confirmPassword = ''
    AgentSignUp.name = ''
  },

  invalidFields: () => {
    if (!AgentSignUp.username) {
      return true
    }
    if (AgentSignUp.password !== AgentSignUp.confirmPassword) {
      return true
    }

    if (!AgentSignUp.name) {
      return true
    }

    return false
  }
}

/**
 * Agent Sign Up form component
 */
const AgentSignUpForm = {
  oninit() {
    AgentSignUp.clear()
  },
  view() {
    return [
      m('h2', 'Sign Up'),
      m('.form', [
        AgentSignUp.errorMsg ? m('p.text-danger', AgentSignUp.errorMsg) : null,

        inputField('username', 'Email', AgentSignUp.username, AgentSignUp.setUsername),
        inputField('password', 'Password', AgentSignUp.password, AgentSignUp.setPassword, 'password'),
        inputField('confirmPassword',
          'Confirm Password',
          AgentSignUp.confirmPassword,
          AgentSignUp.setConfirmPassword,
          'password'),

        inputField('name', 'Name', AgentSignUp.name, AgentSignUp.setName),

        m('.row',
          m("button.btn.btn-primary",
            {
              onclick: AgentSignUp.submit,
              disabled: AgentSignUp.submitting || AgentSignUp.invalidFields(),
            }, "Sign Up")),
      ])
    ]
  }
}

module.exports = {
  SignInForm,
  AuthedComponent,
  AgentSignUpForm
}
