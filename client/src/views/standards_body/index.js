'use strict'

const m = require('mithril')
const AuthService = require('App/services/auth')
const { AuthedComponent } = require('App/views/common/auth')

const _navLink = (route, label) =>
  m('li.nav-item.standards_body_nav',
    { class: m.route.get() === route ? 'active' : '' },
    m(`a.nav-link.standards_body_nav_link[href=${route}]`, { oncreate: m.route.link, }, label))

const _greeting = () =>
  'Welcome, Standards Body team member!'

const _authButtons = () => {
  if (AuthService.isSignedIn()) {
    return m('li.nav-item',
      m(`a.nav-link[href=/index_standards_body.html].standards_body_nav_link#sign_out`, {
        onclick: () => {
          AuthService.clear()
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

const App = {
  _viewName: 'App',
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
                    'img.org-logo[src="/assets/images/pencil.svg"].d-inline-block.align-top'
                  )
                )
              ]
            ),
            m('span.ml-3.greeting_text', _greeting()),
            m('div.collapse.navbar-collapse', [
              m('ul.navbar-nav.ml-auto',
                [
                  m(AuthedComponent, _navLink('/profile', 'My Profile')),
                  m(AuthedComponent, _navLink('/standardsCreate', 'New Standard')),
                  m(AuthedComponent, _navLink('/standardsList', 'View Standards')),
                  m(AuthedComponent, _navLink('/certifyingBodyList', 'View Certifying Bodies')),
                  _authButtons()
                ]),
            ])
          ]),
        m('main.container', { role: 'main' }, [vnode.children]),
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
      m('div.landing-page.landing-page-standards-body', [
        m('div.landing-page-info', [
          m('p.landing-page-info-section.landing-page-info-header', [
            'Use ',
            m('strong', 'ConsenSource'),
            ' to help retailers and brands ensure a transparent and responsible supply chain, from source to shelf'
          ]),
          m('ul.landing-page-info-section', [
            m('li', 'Search for a variety of certified factories'),
            m('li', 'View important certifications, licenses, and contact information for each retailer'),
            m('li', 'Rest assured that both past and current data are accurate, verified, and up-to-date')
          ]),
          m('button.btn.landing-page-action-btn', 'Start the search for certified factories')
        ])
      ])
    ]
}

module.exports = {
  App, Welcome
}
