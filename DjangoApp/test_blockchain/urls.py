from . import views
from django.urls import path

urlpatterns = [
    path('init_org_pool/', views.init_org_pool, name='init_org_pool'),
    path('add_org/', views.add_org, name='add_org'),
    path('remove_org/', views.remove_org, name='remove_org'),
    path('create_event/', views.create_event, name='create_event'),
    path('init_event_reg/', views.init_event_reg, name='init_event_reg'),
    path('close_account/', views.close_account, name='close_account'),
    path('explore_events/', views.explore_events, name='explore_events'),
    path('manage_events/', views.manage_events, name='manage_events'),
    path('init_dictionary/', views.init_dictionary, name='init_dictionary'),
    path('init', views.init, name='init'),
    path('user_event/', views.user_event, name='user_event'),
    path('event_detail/<str:event_id>/', views.event_detail, name='event_detail'),
    path('test_mint/', views.test_mint, name='test_mint'),
    path("generate_metadata/", views.generate_metadata, name="generate_metadata"),
    path('add_validators/', views.add_validators, name='add_validators'),
    path('deactivate_ticket/', views.deactivate_ticket, name='deactivate_ticket'),
    path('activate_ticket/', views.activate_ticket, name='activate_ticket'),
]