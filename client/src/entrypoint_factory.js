'use strict'

const m = require('mithril')

const { SignInForm } = require('App/views/common/auth')
const {App, Welcome} = require('App/views/factory')
const {FactorySignUpForm, FactoryDetails} = require('App/views/factory/details')
const AuthService = require('App/services/auth')
const { ListCertifications } = require('App/views/factory/requests')

AuthService.namespace = 'factory'

let element = document.getElementById("app")
m.route(element, '/', {
    '/': App.subpage(Welcome),
    '/signUp': App.subpage(FactorySignUpForm),
    '/signIn': App.subpage(SignInForm),
    '/profile':  App.subpage(FactoryDetails),
    '/requests':  App.subpage(ListCertifications)

})
