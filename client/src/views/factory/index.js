'use strict'

const m = require('mithril')
const authService = require('App/services/auth')
const agentService = require('App/services/agent')
const modals = require('App/components/modals')
const { AuthedComponent } = require('App/views/common/auth')


const _navLink = (route, asset_active, asset_inactive, label) =>
  m('li.nav-item.retailer_nav',

    m(`a.nav-link.retailer_nav_link[href=${route}]`,
      { class: m.route.get() === route ? 'active' : '', oncreate: m.route.link },
      [m(`img.nav_icon[src=/assets/images/${m.route.get() === route ? asset_active : asset_inactive}]`), m('span.nav_label.p-1.ml-1', label)]))

const _authButtons = () => {
  if (authService.isSignedIn()) {
    return m('li.nav-item',
      m(`a.nav-link[href=/index_factory.html].retailer_nav_link#sign_out`, {
        onclick: () => {
          authService.clear()
          m.route.set('/')
        }
      }, m('img.nav_icon.mr-1[src=/assets/images/logout-icon.svg]'), 'Log Out'))


  } else {
    return [
      m('a.btn.btn-outline-success[href=/signIn]', { oncreate: m.route.link }, 'Sign In'),
      m('a.btn.btn-link.small.text-muted[href=/signUp]', { oncreate: m.route.link }, 'Not a member? Sign Up')
    ]
  }
}

const _greeting = (vnode) => {
  if (vnode.state.agent) {
    return m(AuthedComponent, `Hi, ${vnode.state.agent.name}`)
  }
  return `Welcome, factory team member!`
}

const _getAgentData = (vnode) => authService.getUserData()
  .then((user) => Promise.all([agentService.fetchAgent(user.public_key)])
    .then(([agent]) => {
      vnode.state.agent = agent.data
      vnode.state.loading = false
      m.redraw()
    })
    .catch((e) => {
      console.log(e)
      vnode.state.loading = false
    }))

const App = {
  _viewName: 'App',
  oninit: (vnode) => {
    vnode.state.agent = null
    vnode.state.loading = false

  },
  onupdate: (vnode) => {
    if (authService.isSignedIn() && vnode.state.agent === null && vnode.state.loading === false) {
      _getAgentData(vnode)
    }
  },
  view: (vnode) => {
    if (vnode.state.loading) {
      return [m('.row', 'Loading...')]
    } else {
      return [
        m('nav.navbar.navbar-expand-md.navbar-light.bg-light',
          [
            m(
              "a.navbar-brand.org-brand.greeting_text[href=/]",
              { oncreate: m.route.link },
              [
                m(
                  "span.logo-circle",
                  m(
                    'img.org-logo[src="/assets/images/all-factories-active.svg"].d-inline-block.align-top'
                  )
                )
              ]
            ),
            m('span.ml-3.greeting_text', _greeting(vnode)),
            m('div.collapse.navbar-collapse', [
              m('ul.navbar-nav.ml-auto',
                [
                  m(AuthedComponent, _navLink('/requests', 'certified-factories-icon.svg', 'inactive-cert-factories.svg', 'Request Certification')),
                  m(AuthedComponent, _navLink('/profile', 'active-profile.svg', 'profile-icon.svg', 'Profile')),
                  _authButtons()
                ]),

            ])
          ]),
        m('main.container.mt-5', { role: 'main' }, [vnode.children]),
        m(modals.ModalContainer, { show: modals.displayModal() })
      ]
    }
  },

  subpage: (element) => ({
    onmatch: (_args, _requestedPath) => element,
    render: (vnode) => m(App, vnode)
  })
}

const Welcome = {
  _viewName: 'Welcome',
  view: () =>
    [
      m('div.landing-page.landing-page-factory', [
        m('div.landing-page-info', [
          m('p.landing-page-info-section.landing-page-info-header', [
            'Signal your commitment to sustainable practices by using ',
            m('strong', 'ConsenSource'),
            ' to request audits and certifications for your factory'
          ]),
          m('ul.landing-page-info-section', [
            m('li', 'Initiate requests to be audited and certified'),
            m('li', 'Broadcast your certifications and licenses to interested retailers'),
            m('li', 'Withdraw certification requests at any time and for any reason')
          ]),
          m('button.btn.landing-page-action-btn', 'Open a new certification request')
        ])
      ])
    ],
}
module.exports = {
  App, Welcome
}
